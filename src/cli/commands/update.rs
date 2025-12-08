use crate::clients::HelmClient;
use crate::config::{Config, RepoType};
use crate::error::VesshelmError;
use crate::util::ui;
use console::style;
use semver::Version;
use tokio::sync::Semaphore;
use tokio::task::JoinSet;

use std::sync::Arc;

use super::sync;
use super::{SyncArgs, UpdateArgs};
use crate::util::config_updater::ConfigUpdater;

/// Information about a chart that has a newer version available.
#[derive(Debug, Clone)]
pub struct OutdatedChart {
    pub name: String,
    pub current: String,
    pub latest: String,
}

/// Run repo registration + version checks and return the list of outdated charts.
/// Prints the "checking <chart>... Outdated/Up to date/Skipped" lines and a final
/// summary table when `no_progress` is false.
pub async fn check_outdated_charts(
    config_path: &std::path::Path,
    helm_client: impl HelmClient + Clone + Send + Sync + 'static,
    jobs: usize,
    charts_filter: Option<&[String]>,
    no_progress: bool,
) -> Result<Vec<OutdatedChart>, VesshelmError> {
    let mut config = Config::load_from_path(config_path)?;

    ensure_helm_repos(&config, &helm_client, no_progress)?;

    let cache_minutes = config
        .vesshelm
        .as_ref()
        .map(|v| v.repo_update_cache_minutes)
        .unwrap_or(crate::config::DEFAULT_REPO_UPDATE_CACHE_MINUTES);
    let cache_path = crate::util::repo_update_cache::cache_path_for(config_path);
    update_helm_repos(&helm_client, no_progress, cache_minutes, &cache_path)?;

    if !no_progress {
        ui::print_step("Calculating dependency graph");
    }

    let mut stats = CheckUpdateStats::default();
    let mut outdated: Vec<OutdatedChart> = Vec::new();

    // Sort charts alphabetically for consistent output (initial submission order)
    config.charts.sort_by(|a, b| a.name.cmp(&b.name));

    let semaphore = Arc::new(Semaphore::new(std::cmp::max(1, jobs)));
    let mut handlers: JoinSet<Result<CheckResult, ()>> = JoinSet::new();

    let config_arc = Arc::new(config.clone());

    for chart in &config.charts {
        if let Some(filter) = charts_filter
            && !filter.contains(&chart.name)
        {
            continue;
        }

        let chart = chart.clone();
        let config_ref = config_arc.clone();
        let semaphore_clone = semaphore.clone();
        let helm_client_clone = helm_client.clone();

        let fut = async move {
            let _permit = semaphore_clone.acquire().await.unwrap();

            // Skip non-Helm/OCI charts logic
            let repo_type = if let Some(repo_name) = &chart.repo_name {
                config_ref
                    .repositories
                    .iter()
                    .find(|r| r.name == *repo_name)
                    .map(|r| &r.r#type)
            } else {
                None
            };

            match repo_type {
                Some(RepoType::Helm) => {
                    let chart_ref = if let Some(repo_name) = &chart.repo_name {
                        format!("{}/{}", repo_name, chart.name)
                    } else {
                        chart.name.clone()
                    };

                    match get_latest_version(&chart_ref, &helm_client_clone) {
                        Ok(latest_version_str) => Ok(CheckResult::Checked {
                            chart_name: chart.name.clone(),
                            current_version: chart.version.clone(),
                            latest_version: latest_version_str,
                        }),
                        Err(e) => Ok(CheckResult::Error {
                            chart_name: chart.name.clone(),
                            error: e.to_string(),
                        }),
                    }
                }
                Some(RepoType::Oci) => {
                    let chart_ref = if let Some(repo_name) = &chart.repo_name {
                        if let Some(repo) = config_ref
                            .repositories
                            .iter()
                            .find(|r| r.name == *repo_name)
                        {
                            crate::util::oci_url::build_oci_chart_ref(&repo.url, &chart.name)
                        } else {
                            chart.name.clone()
                        }
                    } else {
                        chart.name.clone()
                    };

                    match get_latest_version_oci(&chart_ref, &chart.name, &helm_client_clone).await
                    {
                        Ok(latest_version_str) => Ok(CheckResult::Checked {
                            chart_name: chart.name.clone(),
                            current_version: chart.version.clone(),
                            latest_version: latest_version_str,
                        }),
                        Err(e) => Ok(CheckResult::Error {
                            chart_name: chart.name.clone(),
                            error: e.to_string(),
                        }),
                    }
                }
                _ => Ok(CheckResult::Skipped {
                    chart_name: chart.name.clone(),
                    reason: "local/git".to_string(),
                }),
            }
        };

        if jobs <= 1 {
            // Sequential
            if let Ok(res) = fut.await {
                handle_check_result(res, &mut outdated, &mut stats, no_progress)
            }
        } else {
            handlers.spawn(fut);
        }
    }

    if jobs > 1 {
        while let Some(res) = handlers.join_next().await {
            if let Ok(Ok(result)) = res {
                handle_check_result(result, &mut outdated, &mut stats, no_progress)
            }
        }
    }

    if !no_progress {
        crate::util::summary::Summary::new()
            .add("Outdated:", style(stats.outdated).yellow().bold())
            .add("Up to date:", style(stats.up_to_date).green().bold())
            .add("Skipped:", style(stats.skipped).dim().bold())
            .add("Failed:", style(stats.failed).red().bold())
            .print();
    }

    // Parallel join order is undefined, so sort for stable downstream UX (selectors).
    outdated.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(outdated)
}

/// Apply the given chart version bumps to `vesshelm.yaml`. Logs progress to stdout
/// when `no_progress` is false.
pub fn apply_chart_updates(
    config_path: &std::path::Path,
    charts: &[OutdatedChart],
    no_progress: bool,
) -> Result<(), VesshelmError> {
    if !no_progress {
        println!("\n{} Applying updates...", style("📝").bold());
    }
    for chart in charts {
        if let Err(e) = ConfigUpdater::update_chart_version(config_path, &chart.name, &chart.latest)
        {
            println!(
                "{} Failed to update {}: {}",
                style("WARN:").yellow().bright(),
                chart.name,
                e
            );
        } else if !no_progress {
            println!(
                "Updated {} to {}",
                style(&chart.name).bold(),
                style(&chart.latest).green().bright()
            );
        }
    }
    if !no_progress {
        println!("{} vesshelm.yaml updated.", style("✅").green().bright());
    }
    Ok(())
}

pub async fn run(
    args: UpdateArgs,
    no_progress: bool,
    config_path: &std::path::Path,
    helm_client: impl HelmClient + Clone + Send + Sync + 'static,
) -> Result<(), VesshelmError> {
    let outdated = check_outdated_charts(
        config_path,
        helm_client,
        args.jobs,
        args.charts.as_deref(),
        no_progress,
    )
    .await?;

    let updates_found = !outdated.is_empty();
    let should_apply = args.apply || args.apply_sync;

    match (should_apply, updates_found) {
        (true, true) => {
            apply_chart_updates(config_path, &outdated, no_progress)?;

            if args.apply_sync {
                println!();
                let sync_args = SyncArgs {
                    charts: args.charts,
                    ignore_skip: false,
                    jobs: args.jobs,
                };
                sync::run(sync_args, no_progress, config_path).await?;
            }
        }
        (false, true) => {
            if !no_progress {
                println!(
                    "\nRun with {} to apply changes or {} to apply and sync.",
                    style("--apply").cyan().bright(),
                    style("--apply-sync").cyan().bright()
                );
            }
        }
        _ => {
            if !no_progress {
                println!("\nAll checked charts are up to date.");
            }
        }
    }

    Ok(())
}

enum CheckResult {
    Checked {
        chart_name: String,
        current_version: Option<String>,
        latest_version: String,
    },
    Skipped {
        chart_name: String,
        reason: String,
    },
    Error {
        chart_name: String,
        error: String,
    },
}

#[derive(Default)]
struct CheckUpdateStats {
    outdated: u32,
    up_to_date: u32,
    skipped: u32,
    failed: u32,
}

fn handle_check_result(
    result: CheckResult,
    outdated: &mut Vec<OutdatedChart>,
    stats: &mut CheckUpdateStats,
    no_progress: bool,
) {
    match result {
        CheckResult::Checked {
            chart_name,
            current_version,
            latest_version,
        } => {
            if !no_progress {
                print!("checking {}... ", chart_name);
            }
            check_update(
                &chart_name,
                current_version.as_deref(),
                &latest_version,
                outdated,
                stats,
                no_progress,
            );
        }
        CheckResult::Skipped { chart_name, reason } => {
            if !no_progress {
                print!("checking {}... ", chart_name);
                println!("{}", style(format!("Skipped ({})", reason)).dim());
            }
            stats.skipped += 1;
        }
        CheckResult::Error { chart_name, error } => {
            if !no_progress {
                print!("checking {}... ", chart_name);
                println!(
                    "{} Failed to fetch version: {}",
                    style("Error").red().bright(),
                    error
                );
            }
            stats.failed += 1;
        }
    }
}

fn ensure_helm_repos(
    config: &Config,
    helm_client: &impl HelmClient,
    no_progress: bool,
) -> Result<(), VesshelmError> {
    if !no_progress {
        println!(
            "{} Ensuring repositories are registered...",
            style("📦").dim()
        );
    }
    for repo in &config.repositories {
        if repo.r#type == RepoType::Helm {
            helm_client.repo_add(&repo.name, &repo.url).map_err(|e| {
                VesshelmError::CommandFailed(format!(
                    "Failed to add helm repository '{}': {}",
                    repo.name, e
                ))
            })?;
        }
    }
    Ok(())
}

fn update_helm_repos(
    helm_client: &impl HelmClient,
    no_progress: bool,
    cache_minutes: u64,
    cache_path: &std::path::Path,
) -> Result<(), VesshelmError> {
    if !crate::util::repo_update_cache::should_update(cache_path, cache_minutes) {
        if !no_progress {
            let age =
                crate::util::repo_update_cache::last_update_age_minutes(cache_path).unwrap_or(0);
            println!(
                "{} Helm repositories update skipped (cached, {}m ago)",
                style("⏭").dim(),
                age
            );
        }
        return Ok(());
    }
    if !no_progress {
        println!("{} Updating Helm repositories...", style("🔄").dim());
    }
    helm_client.repo_update().map_err(|e| {
        VesshelmError::CommandFailed(format!("Failed to update helm repositories: {}", e))
    })?;
    crate::util::repo_update_cache::mark_updated(cache_path);
    Ok(())
}

// OciError removed, use VesshelmError

use crate::clients::OciClient;

async fn get_latest_version_oci(
    chart_ref: &str,
    chart_name: &str,
    helm_client: &impl HelmClient,
) -> std::result::Result<String, VesshelmError> {
    // "helm show chart <ref>" returns the Chart.yaml content
    match helm_client.show_chart(chart_ref) {
        Ok(stdout) => {
            // Parse YAML output
            let chart_yaml: serde_yaml_ng::Value =
                serde_yaml_ng::from_str(&stdout).map_err(|e| {
                    VesshelmError::Config(crate::config::error::ConfigError::ParseFailed(
                        e,
                        format!("helm show chart {}", chart_ref),
                    ))
                })?;

            if let Some(version_str) = chart_yaml["version"].as_str() {
                return Ok(version_str.to_string());
            }
            // No version field — fall through to OCI client
        }
        Err(_) => {
            // helm show chart failed — fall through to OCI client
        }
    }

    // Fallback: Try Native OCI Client
    // chart_ref is e.g. "oci://ghcr.io/hyperspike/valkey-operator"

    let client = OciClient::new();
    match client.get_tags(chart_ref, chart_name).await {
        Ok(tags) => {
            // Find max version
            let mut best_version: Option<(Version, String)> = None;
            for tag in tags {
                if let Ok(semver) = parse_version(&tag) {
                    if let Some((max, _)) = &best_version {
                        if semver > *max {
                            best_version = Some((semver, tag));
                        }
                    } else {
                        best_version = Some((semver, tag));
                    }
                }
            }

            if let Some((_, tag)) = best_version {
                Ok(tag)
            } else {
                Err(VesshelmError::CommandFailed(
                    "No valid semantic version tags found".to_string(),
                ))
            }
        }
        Err(e) => {
            // Map Client Error
            let client_err = crate::clients::error::ClientError::from(e);
            Err(VesshelmError::from(client_err))
        }
    }
}

fn check_update(
    chart_name: &str,
    current_version_str: Option<&str>,
    latest_version_str: &str,
    outdated: &mut Vec<OutdatedChart>,
    stats: &mut CheckUpdateStats,
    no_progress: bool,
) {
    let current_version_str = current_version_str.unwrap_or("unknown");

    let current_semver = parse_version(current_version_str);
    let latest_semver = parse_version(latest_version_str);

    let (is_outdated, suffix) = match (current_semver, latest_semver) {
        (Ok(current), Ok(latest)) => (latest > current, ""),
        _ => (
            latest_version_str != current_version_str,
            " (semver parse failed)",
        ),
    };

    if is_outdated {
        if !no_progress {
            println!(
                "{} {} -> {}{}",
                style("Outdated").yellow().bright(),
                style(current_version_str).dim(),
                style(latest_version_str).green().bright(),
                suffix
            );
        }
        outdated.push(OutdatedChart {
            name: chart_name.to_string(),
            current: current_version_str.to_string(),
            latest: latest_version_str.to_string(),
        });
        stats.outdated += 1;
    } else {
        if !no_progress {
            println!("{}", style("Up to date").green().bright());
        }
        stats.up_to_date += 1;
    }
}

fn get_latest_version(
    chart_ref: &str,
    helm_client: &impl HelmClient,
) -> Result<String, VesshelmError> {
    let try_parse = |yaml_str: &str| -> Result<Option<String>, VesshelmError> {
        let versions: Vec<serde_yaml_ng::Value> =
            serde_yaml_ng::from_str(yaml_str).map_err(|e| {
                VesshelmError::Config(crate::config::error::ConfigError::ParseFailed(
                    e,
                    format!("helm search repo {}", chart_ref),
                ))
            })?;

        if let Some(chart_value) = versions.first() {
            if let Some(version_str) = chart_value["version"].as_str() {
                Ok(Some(version_str.to_string()))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    };

    // Try stable search first
    if let Ok(yaml) = helm_client.search_repo(chart_ref, false)
        && let Ok(Some(v)) = try_parse(&yaml)
    {
        return Ok(v);
    }

    // Fallback to --devel
    let yaml_devel = helm_client.search_repo(chart_ref, true).map_err(|e| {
        VesshelmError::CommandFailed(format!("helm search failed (even with --devel): {}", e))
    })?;

    if let Ok(Some(v)) = try_parse(&yaml_devel) {
        return Ok(v);
    }

    // If still nothing
    Err(VesshelmError::Config(
        crate::config::error::ConfigError::ChartNotFound(chart_ref.to_string()),
    ))
}

fn parse_version(v: &str) -> Result<Version, semver::Error> {
    Version::parse(v.strip_prefix('v').unwrap_or(v))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_version() {
        assert_eq!(parse_version("1.2.3").unwrap(), Version::new(1, 2, 3));
        assert_eq!(parse_version("v1.2.3").unwrap(), Version::new(1, 2, 3));
        assert!(parse_version("invalid").is_err());
    }

    #[test]
    fn test_version_comparison() {
        let v1 = parse_version("1.18.4").unwrap();
        let v2 = parse_version("1.19.0-pre.3").unwrap();
        // 1.19.0-pre.3 is NEWER than 1.18.4.
        // So v2 > v1.
        assert!(v2 > v1);

        // User scenario: "Outdated 1.19.0-pre.3 -> 1.18.4"
        // Current: 1.19.0-pre.3 (v2)
        // Latest (from repo): 1.18.4 (v1)
        // Check condition: if latest > current
        // if v1 > v2 { update } else { no update }
        assert!(v1 <= v2); // Should NOT suggest update

        // User scenario: "Outdated 1.19.2 -> v1.19.2"
        let v3 = parse_version("1.19.2").unwrap();
        let v4 = parse_version("v1.19.2").unwrap();
        assert_eq!(v3, v4);
        assert!(v3 <= v4); // Should NOT suggest update
    }
}
