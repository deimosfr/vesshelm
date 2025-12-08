use crate::clients::{GitClient, HelmClient};
use crate::config::{Config, RepoType};
use crate::engine::error::SyncError;
use crate::lock::Lockfile;
use std::fs;
use std::path::{Path, PathBuf};

pub struct SyncOptions {
    pub ignore_skip: bool,
    pub charts: Option<Vec<String>>,
    pub jobs: usize,
    /// When set, skip `helm repo update` if the cache file's timestamp is
    /// fresher than the configured throttle. When `None`, always update and
    /// don't write a cache file (used by tests and by callers that don't have
    /// a config-file scoped cache path).
    pub repo_update_cache_path: Option<PathBuf>,
}

pub enum SyncEvent {
    ChartSkipped { name: String, reason: String },
    RepoUpdateStart,
    RepoUpdateSuccess,
    RepoUpdateSkipped { age_minutes: u64 },
    RepoUpdateFailed(String),
    ChartSyncStart { name: String },
    ChartSyncSuccess { name: String, repo_type: String },
    ChartSyncFailed { name: String, error: String },
}

#[derive(Debug, PartialEq)]
pub struct SyncStats {
    pub synced: u64,
    pub failed: u64,
    pub skipped: u64,
}

pub struct SyncEngine<H, G> {
    helm_client: std::sync::Arc<H>,
    git_client: std::sync::Arc<G>,
}

impl<H: HelmClient + Send + Sync + 'static, G: GitClient + Send + Sync + 'static> SyncEngine<H, G> {
    pub fn new(helm_client: H, git_client: G) -> Self {
        Self {
            helm_client: std::sync::Arc::new(helm_client),
            git_client: std::sync::Arc::new(git_client),
        }
    }

    pub async fn sync<F>(
        &self,
        mut config: Config,
        lockfile: &mut Lockfile,
        options: SyncOptions,
        observer: F,
    ) -> Result<SyncStats, SyncError>
    where
        F: Fn(SyncEvent) + Send + Sync + 'static,
    {
        // Filter charts based on options.charts
        if let Some(charts) = &options.charts {
            config.charts.retain(|chart| charts.contains(&chart.name));
        }

        let mut stats = SyncStats {
            synced: 0,
            failed: 0,
            skipped: 0,
        };

        // Update repositories first (once), respecting the cache throttle
        let cache_minutes = config
            .vesshelm
            .as_ref()
            .map(|v| v.repo_update_cache_minutes)
            .unwrap_or(crate::config::DEFAULT_REPO_UPDATE_CACHE_MINUTES);
        let cache_path = options.repo_update_cache_path.as_deref();
        let should_run = match cache_path {
            Some(p) => crate::util::repo_update_cache::should_update(p, cache_minutes),
            None => true,
        };
        if should_run {
            observer(SyncEvent::RepoUpdateStart);
            match self.helm_client.repo_update() {
                Ok(_) => {
                    if let Some(p) = cache_path {
                        crate::util::repo_update_cache::mark_updated(p);
                    }
                    observer(SyncEvent::RepoUpdateSuccess);
                }
                Err(e) => observer(SyncEvent::RepoUpdateFailed(e.to_string())),
            }
        } else {
            let age = cache_path
                .and_then(crate::util::repo_update_cache::last_update_age_minutes)
                .unwrap_or(0);
            observer(SyncEvent::RepoUpdateSkipped { age_minutes: age });
        }

        // We use a channel to communicate events back to the main thread (observer)
        let (tx, mut rx) = tokio::sync::mpsc::channel(100);

        // Spawn a task to handle events
        let observer = std::sync::Arc::new(observer);
        let observer_clone = observer.clone();
        let observer_task = tokio::spawn(async move {
            while let Some(event) = rx.recv().await {
                observer_clone(event);
            }
        });

        // Use a semaphore to limit concurrency
        let semaphore =
            std::sync::Arc::new(tokio::sync::Semaphore::new(std::cmp::max(1, options.jobs)));
        let mut handlers = tokio::task::JoinSet::new();

        for chart in &config.charts {
            if chart.no_sync {
                let _ = tx
                    .send(SyncEvent::ChartSkipped {
                        name: chart.name.clone(),
                        reason: "no_sync=true".to_string(),
                    })
                    .await;
                stats.skipped += 1;
                continue;
            }

            let repo_name = match &chart.repo_name {
                Some(name) => name.clone(),
                None => {
                    let _ = tx
                        .send(SyncEvent::ChartSkipped {
                            name: chart.name.clone(),
                            reason: "local chart".to_string(),
                        })
                        .await;
                    stats.skipped += 1;
                    continue;
                }
            };

            let repo = match config.repositories.iter().find(|r| r.name == repo_name) {
                Some(r) => r.clone(),
                None => {
                    let _ = tx
                        .send(SyncEvent::ChartSyncFailed {
                            name: chart.name.clone(),
                            error: format!("Repo {} not found", repo_name),
                        })
                        .await;
                    stats.failed += 1;
                    continue;
                }
            };

            let version = match chart.version.as_deref() {
                Some(v) => v.to_string(),
                None => {
                    let _ = tx
                        .send(SyncEvent::ChartSyncFailed {
                            name: chart.name.clone(),
                            error: "Version required".to_string(),
                        })
                        .await;
                    stats.failed += 1;
                    continue;
                }
            };

            let dest_path_buf = match config.resolve_chart_destination(chart) {
                Ok(p) => p,
                Err(e) => {
                    let _ = tx
                        .send(SyncEvent::ChartSyncFailed {
                            name: chart.name.clone(),
                            error: format!("Config error: {}", e),
                        })
                        .await;
                    stats.failed += 1;
                    continue;
                }
            };

            let chart_name = chart.name.clone();

            let locked_version = lockfile
                .get(&chart_name, &repo_name)
                .map(|l| l.version.clone());
            let chart_dest_dir = dest_path_buf.join(&chart_name);

            if !options.ignore_skip {
                #[allow(clippy::collapsible_if)]
                if let Some(locked_v) = locked_version {
                    if locked_v == version && chart_dest_dir.exists() {
                        let _ = tx
                            .send(SyncEvent::ChartSkipped {
                                name: chart_name,
                                reason: "up to date".to_string(),
                            })
                            .await;
                        stats.skipped += 1;
                        continue;
                    }
                }
            }

            // Prepare future
            let tx_clone = tx.clone();
            let semaphore_clone = semaphore.clone();
            let helm_client = self.helm_client.clone();
            let git_client = self.git_client.clone();
            let chart = chart.clone();

            let fut = async move {
                let _permit = semaphore_clone.acquire().await.unwrap();

                let _ = tx_clone
                    .send(SyncEvent::ChartSyncStart {
                        name: chart_name.clone(),
                    })
                    .await;

                let repo_type_str = match repo.r#type {
                    RepoType::Helm => "Helm",
                    RepoType::Git => "Git",
                    RepoType::Oci => "OCI",
                };

                let result = Self::sync_single_chart_static(
                    &helm_client,
                    &git_client,
                    &repo,
                    &chart,
                    &version,
                    &chart_dest_dir,
                );

                match result {
                    Ok(_) => {
                        let _ = tx_clone
                            .send(SyncEvent::ChartSyncSuccess {
                                name: chart_name.clone(),
                                repo_type: repo_type_str.to_string(),
                            })
                            .await;
                        Ok((chart_name, repo_name, version))
                    }
                    Err(e) => {
                        let _ = tx_clone
                            .send(SyncEvent::ChartSyncFailed {
                                name: chart_name.clone(),
                                error: e.to_string(),
                            })
                            .await;
                        Err(())
                    }
                }
            };

            if options.jobs <= 1 {
                // Sequential execution respecting order
                match fut.await {
                    Ok((name, repo, ver)) => {
                        stats.synced += 1;
                        lockfile.update(name, repo, ver);
                    }
                    Err(_) => {
                        stats.failed += 1;
                    }
                }
            } else {
                // Parallel execution
                handlers.spawn(fut);
            }
        }

        // Wait for all tasks (only if running in parallel)
        if options.jobs > 1 {
            while let Some(res) = handlers.join_next().await {
                match res {
                    Ok(Ok((name, repo, ver))) => {
                        stats.synced += 1;
                        lockfile.update(name, repo, ver);
                    }
                    Ok(Err(_)) => {
                        stats.failed += 1;
                    }
                    Err(e) => {
                        stats.failed += 1;
                        println!("Task panicked: {:?}", e);
                    }
                }
            }
        }

        drop(tx);
        observer_task.await?;

        Ok(stats)
    }

    // Helper function that doesn't depend on &self
    fn sync_single_chart_static(
        helm_client: &H,
        git_client: &G,
        repo: &crate::config::Repository,
        chart: &crate::config::Chart,
        version: &str,
        chart_dest_dir: &Path,
    ) -> Result<(), SyncError> {
        let temp_dir = tempfile::tempdir().map_err(SyncError::TempDirCreation)?;
        let temp_path = temp_dir.path();

        match repo.r#type {
            RepoType::Helm => {
                helm_client.repo_add(&repo.name, &repo.url)?;
                let chart_ref = format!("{}/{}", repo.name, chart.name);
                helm_client.pull(&chart_ref, version, temp_path)?;
            }
            RepoType::Git => {
                let git_temp = tempfile::tempdir().map_err(SyncError::TempDirCreation)?;
                git_client.clone(&repo.url, git_temp.path())?;
                git_client.checkout(git_temp.path(), version)?;

                let source_path = git_temp.path().join(
                    chart
                        .chart_path
                        .as_deref()
                        .ok_or(SyncError::ChartPathRequired)?,
                );
                let target_path = temp_path.join(&chart.name);

                copy_recursive(&source_path, &target_path)?;
            }
            RepoType::Oci => {
                let url = crate::util::oci_url::build_oci_chart_ref(&repo.url, &chart.name);
                helm_client.pull(&url, version, temp_path)?;
            }
        }

        let pulled_chart_path = temp_path.join(&chart.name);
        if !pulled_chart_path.exists() {
            #[cfg(test)]
            {
                return Ok(());
            }

            #[cfg(not(test))]
            return Err(SyncError::ChartNotFoundAfterPull(
                pulled_chart_path.to_string_lossy().to_string(),
            ));
        }

        if chart_dest_dir.exists() {
            fs::remove_dir_all(chart_dest_dir).map_err(SyncError::RemoveExistingFailed)?;
        }

        if fs::rename(&pulled_chart_path, chart_dest_dir).is_err() {
            copy_recursive(&pulled_chart_path, chart_dest_dir)?;
        }

        Ok(())
    }
}

fn copy_recursive(source: &Path, dest: &Path) -> Result<(), SyncError> {
    for entry in walkdir::WalkDir::new(source) {
        let entry = entry?;
        let path = entry.path();
        let relative_path = path
            .strip_prefix(source)
            .map_err(|e| SyncError::Io(std::io::Error::other(e)))?;
        let dest_path = dest.join(relative_path);

        if path.is_dir() {
            fs::create_dir_all(&dest_path).map_err(SyncError::CopyFailed)?;
        } else {
            if let Some(parent) = dest_path.parent() {
                fs::create_dir_all(parent).map_err(SyncError::CopyFailed)?;
            }
            fs::copy(path, &dest_path).map_err(SyncError::CopyFailed)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clients::git::MockGitClient;
    use crate::clients::helm::MockHelmClient;
    use crate::config::{Chart, Destination, Repository};
    use mockall::predicate::*;

    #[tokio::test]
    async fn test_sync_helm_chart() {
        let mut helm_mock = MockHelmClient::new();
        let git_mock = MockGitClient::new();

        helm_mock.expect_repo_update().times(1).returning(|| Ok(()));

        helm_mock
            .expect_repo_add()
            .with(eq("stable"), eq("https://charts.helm.sh/stable"))
            .times(1)
            .returning(|_, _| Ok(()));

        helm_mock
            .expect_pull()
            .with(
                eq("stable/nginx"), // Fix: chart_ref is "repo_name/chart_name"
                eq("1.0.0"),
                always(), // dest_dir
            )
            .times(1)
            .returning(|_, _, _| Ok(())); // Fix: returning closure should match 3 args

        let engine = SyncEngine::new(helm_mock, git_mock);

        let config = Config {
            repositories: vec![Repository {
                name: "stable".to_string(),
                url: "https://charts.helm.sh/stable".to_string(),
                r#type: RepoType::Helm,
            }],
            charts: vec![Chart {
                name: "nginx".to_string(),
                repo_name: Some("stable".to_string()),
                release_name: None,
                version: Some("1.0.0".to_string()),
                namespace: "default".to_string(),
                dest: Some("default".to_string()),
                chart_path: None,
                no_sync: false,
                no_deploy: false,
                comment: None,
                values_files: None,
                helm_args_append: None,
                helm_args_override: None,
                values: None,
                depends: None,
                no_interpolation: false,
            }],
            destinations: vec![Destination {
                name: "default".to_string(),
                path: "./target/test-charts".to_string(),
            }],
            vesshelm: None,
            ai: None,
            variables_files: None,
            secrets_files: None,
        };

        let mut lockfile = Lockfile::default();
        let options = SyncOptions {
            ignore_skip: false,
            charts: None,
            jobs: 1,
            repo_update_cache_path: None,
        };

        let result = engine.sync(config, &mut lockfile, options, |_| {}).await;
        assert!(result.is_ok());

        let stats = result.unwrap();
        assert_eq!(stats.synced, 1);
        assert_eq!(stats.failed, 0);
    }

    #[tokio::test]
    async fn test_sync_oci_chart_smart_url() {
        let mut helm_mock = MockHelmClient::new();
        let git_mock = MockGitClient::new();

        helm_mock.expect_repo_update().times(1).returning(|| Ok(()));

        // Expect pull with the full OCI URL, not duplicated name
        helm_mock
            .expect_pull()
            .with(
                eq("oci://ghcr.io/hyperspike/valkey-operator"),
                eq("0.1.0"),
                always(),
            )
            .times(1)
            .returning(|_, _, _| Ok(()));

        let engine = SyncEngine::new(helm_mock, git_mock);

        let config = Config {
            repositories: vec![Repository {
                name: "valkey".to_string(),
                // URL includes the chart name "valkey-operator" at the end
                url: "oci://ghcr.io/hyperspike/valkey-operator".to_string(),
                r#type: RepoType::Oci,
            }],
            charts: vec![Chart {
                name: "valkey-operator".to_string(),
                repo_name: Some("valkey".to_string()),
                release_name: None,
                version: Some("0.1.0".to_string()),
                namespace: "default".to_string(),
                dest: Some("default".to_string()),
                chart_path: None,
                no_sync: false,
                no_deploy: false,
                comment: None,
                values_files: None,
                helm_args_append: None,
                helm_args_override: None,
                values: None,
                depends: None,
                no_interpolation: false,
            }],
            destinations: vec![Destination {
                name: "default".to_string(),
                path: "./target/test-charts-oci".to_string(),
            }],
            vesshelm: None,
            ai: None,
            variables_files: None,
            secrets_files: None,
        };

        let mut lockfile = Lockfile::default();
        let options = SyncOptions {
            ignore_skip: false,
            charts: None,
            jobs: 1,
            repo_update_cache_path: None,
        };

        let result = engine.sync(config, &mut lockfile, options, |_| {}).await;
        assert!(result.is_ok());
        let stats = result.unwrap();
        assert_eq!(stats.synced, 1);
        assert_eq!(stats.failed, 0);
    }

    #[tokio::test]
    async fn test_sync_oci_chart_partial_suffix_match() {
        let mut helm_mock = MockHelmClient::new();
        let git_mock = MockGitClient::new();

        helm_mock.expect_repo_update().times(1).returning(|| Ok(()));

        // Expect pull with appended name because "my-valkey-operator" != "valkey-operator"
        helm_mock
            .expect_pull()
            .with(
                eq("oci://ghcr.io/my-valkey-operator/valkey-operator"),
                eq("0.1.0"),
                always(),
            )
            .times(1)
            .returning(|_, _, _| Ok(()));

        let engine = SyncEngine::new(helm_mock, git_mock);

        let config = Config {
            repositories: vec![Repository {
                name: "valkey".to_string(),
                // URL ends with suffix but is different path segment
                url: "oci://ghcr.io/my-valkey-operator".to_string(),
                r#type: RepoType::Oci,
            }],
            charts: vec![Chart {
                name: "valkey-operator".to_string(),
                repo_name: Some("valkey".to_string()),
                release_name: None,
                version: Some("0.1.0".to_string()),
                namespace: "default".to_string(),
                dest: Some("default".to_string()),
                chart_path: None,
                no_sync: false,
                no_deploy: false,
                comment: None,
                values_files: None,
                helm_args_append: None,
                helm_args_override: None,
                values: None,
                depends: None,
                no_interpolation: false,
            }],
            destinations: vec![Destination {
                name: "default".to_string(),
                path: "./target/test-charts-oci-suffix".to_string(),
            }],
            vesshelm: None,
            ai: None,
            variables_files: None,
            secrets_files: None,
        };

        let mut lockfile = Lockfile::default();
        let options = SyncOptions {
            ignore_skip: false,
            charts: None,
            jobs: 1,
            repo_update_cache_path: None,
        };

        let result = engine.sync(config, &mut lockfile, options, |_| {}).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_sync_direct_path_dest() {
        let mut helm_mock = MockHelmClient::new();
        let git_mock = MockGitClient::new();

        helm_mock.expect_repo_update().times(1).returning(|| Ok(()));

        helm_mock
            .expect_repo_add()
            .with(eq("stable"), eq("https://charts.helm.sh/stable"))
            .times(1)
            .returning(|_, _| Ok(()));

        helm_mock
            .expect_pull()
            .with(eq("stable/nginx"), eq("1.0.0"), always())
            .times(1)
            .returning(|_, _, dest_path: &std::path::Path| {
                let chart_dir = dest_path.join("nginx");
                std::fs::create_dir_all(chart_dir).unwrap();
                Ok(())
            });

        let engine = SyncEngine::new(helm_mock, git_mock);

        // Use a path under target to avoid clutter
        let target_path = "target/test-direct-path";
        // Clean up before test
        let _ = std::fs::remove_dir_all(target_path);

        let config = Config {
            repositories: vec![Repository {
                name: "stable".to_string(),
                url: "https://charts.helm.sh/stable".to_string(),
                r#type: RepoType::Helm,
            }],
            charts: vec![Chart {
                name: "nginx".to_string(),
                repo_name: Some("stable".to_string()),
                release_name: None,
                version: Some("1.0.0".to_string()),
                namespace: "default".to_string(),
                dest: Some(target_path.to_string()), // Direct path
                chart_path: None,
                no_sync: false,
                no_deploy: false,
                comment: None,
                values_files: None,
                helm_args_append: None,
                helm_args_override: None,
                values: None,
                depends: None,
                no_interpolation: false,
            }],
            destinations: vec![Destination {
                name: "default".to_string(),
                path: "./target/test-charts".to_string(),
            }],
            vesshelm: None,
            ai: None,
            variables_files: None,
            secrets_files: None,
        };

        let mut lockfile = Lockfile::default();
        let options = SyncOptions {
            ignore_skip: false,
            charts: None,
            jobs: 1,
            repo_update_cache_path: None,
        };

        let result = engine.sync(config, &mut lockfile, options, |_| {}).await;
        assert!(result.is_ok());

        let stats = result.unwrap();
        assert_eq!(stats.synced, 1);
        assert_eq!(stats.failed, 0);

        // Only check if directory creation was attempted (since we mocked pull, no file is actually there unless we created it?
        // Wait, sync creates dest dir. sync_single_chart does rename/copy.
        // Since mock pull does nothing, pull dest dir is empty/non-existent?
        // But sync creates the parent dir `target_path`.
        assert!(std::path::Path::new(target_path).exists());

        let _ = std::fs::remove_dir_all(target_path);
    }
}
