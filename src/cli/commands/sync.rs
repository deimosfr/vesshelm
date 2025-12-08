use super::SyncArgs;
use crate::clients::git::RealGitClient;
use crate::clients::helm::RealHelmClient;
use crate::config::Config;
use crate::engine::sync::{SyncEngine, SyncEvent, SyncOptions};
use crate::error::VesshelmError;
use crate::lock::Lockfile;
use crate::util::progress::ProgressTracker;
use crate::util::ui;
use console::style;
use std::path::Path;

pub async fn run(
    args: SyncArgs,
    no_progress: bool,
    config_path: &Path,
) -> Result<(), VesshelmError> {
    let mut config = Config::load_from_path(config_path)?;

    // Validate positional charts arguments
    if let Some(charts) = &args.charts {
        let available_names: Vec<_> = config.charts.iter().map(|c| c.name.as_str()).collect();
        crate::util::filter::validate_chart_args(&available_names, charts)?;
    }

    // Sort charts alphabetically by name for deterministic execution order
    config.charts.sort_by(|a, b| a.name.cmp(&b.name));

    // We need to count total charts first for the progress bar
    // This logic duplicates filter logic slightly but is needed for UI
    let total_charts = if let Some(charts) = &args.charts {
        config
            .charts
            .iter()
            .filter(|c| charts.contains(&c.name))
            .count() as u64
    } else {
        config.charts.len() as u64
    };

    if !no_progress {
        ui::print_step(&format!(
            "Syncing {} charts...",
            style(total_charts).bold().bright()
        ));
    }

    let tracker = ProgressTracker::new(total_charts, no_progress)?;
    let lockfile_path = config_path.with_file_name("vesshelm.lock");
    let mut lockfile = Lockfile::load(&lockfile_path).unwrap_or_default();

    let helm_client = RealHelmClient::new();
    let git_client = RealGitClient::new();
    let engine = SyncEngine::new(helm_client, git_client);

    let options = SyncOptions {
        ignore_skip: args.ignore_skip,
        charts: args.charts,
        jobs: args.jobs,
        repo_update_cache_path: Some(crate::util::repo_update_cache::cache_path_for(config_path)),
    };

    let tracker_clone = tracker.clone();

    let stats = engine
        .sync(config, &mut lockfile, options, move |event| match event {
            SyncEvent::ChartSkipped { name, reason } => {
                tracker_clone.println(&format!(
                    " {} {} ({})",
                    style("[SKIP]").dim().bright(),
                    name,
                    reason
                ));
                tracker_clone.inc();
            }
            SyncEvent::RepoUpdateStart => {
                tracker_clone.set_message("Updating Helm repositories...");
            }
            SyncEvent::RepoUpdateSuccess => {}
            SyncEvent::RepoUpdateSkipped { age_minutes } => {
                tracker_clone.println(&format!(
                    " {} Helm repo update skipped (cached, {}m ago)",
                    style("[CACHE]").dim().bright(),
                    age_minutes
                ));
            }
            SyncEvent::RepoUpdateFailed(e) => {
                tracker_clone.println(&format!(
                    " {} Failed to update helm repos: {}",
                    style("WARN:").yellow().bright(),
                    e
                ));
            }
            SyncEvent::ChartSyncStart { name } => {
                tracker_clone.set_message(format!("Syncing {}...", name));
            }
            SyncEvent::ChartSyncSuccess { name, repo_type } => {
                tracker_clone.println(&format!(
                    " {}   {} {}",
                    style("[OK]").green().bright(),
                    name,
                    style(format!("({})", repo_type)).dim().bright()
                ));
                tracker_clone.inc();
            }
            SyncEvent::ChartSyncFailed { name, error } => {
                tracker_clone.println(&format!(
                    " {} {}: {}",
                    style("[FAIL]").red().bright(),
                    name,
                    error
                ));
                tracker_clone.inc();
            }
        })
        .await?;

    if stats.synced > 0
        && let Err(e) = lockfile.save()
    {
        tracker.println(&format!(
            "{} Failed to save lockfile: {}",
            style("WARN:").yellow().bright(),
            e
        ));
    }

    tracker.finish_with_message("Sync completed");

    if !no_progress {
        crate::util::summary::Summary::new()
            .add("Synced:", style(stats.synced).green().bright())
            .add("Failed:", style(stats.failed).red().bright())
            .add("Skipped:", style(stats.skipped).yellow().bright())
            .print();
    }

    if stats.failed > 0 {
        return Err(VesshelmError::CommandFailed(
            "Some charts failed to sync".to_string(),
        ));
    }

    Ok(())
}
