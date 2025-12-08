use super::SyncArgs;
use anyhow::{Context, Result};
use console::style;
use std::path::Path;
use vesshelm::clients::git::RealGitClient;
use vesshelm::clients::helm::RealHelmClient;
use vesshelm::config::Config;
use vesshelm::engine::sync::{SyncEngine, SyncEvent, SyncOptions};
use vesshelm::lock::Lockfile;
use vesshelm::util::progress::ProgressTracker;

pub async fn run(args: SyncArgs, no_progress: bool, config_path: &Path) -> Result<()> {
    let config = Config::load_from_path(config_path)?;

    // Validate --only arguments
    if let Some(only) = &args.only {
        let available_names: Vec<_> = config.charts.iter().map(|c| c.name.as_str()).collect();
        vesshelm::util::filter::validate_only_args(&available_names, only)?;
    }

    // We need to count total charts first for the progress bar
    // This logic duplicates filter logic slightly but is needed for UI
    let total_charts = if let Some(only) = &args.only {
        config
            .charts
            .iter()
            .filter(|c| only.contains(&c.name))
            .count() as u64
    } else {
        config.charts.len() as u64
    };

    println!(
        "{} Syncing {} charts...",
        style("==>").bold().green(),
        style(total_charts).bold()
    );

    let tracker = ProgressTracker::new(total_charts, no_progress)
        .context("Failed to initialize progress tracker")?;
    let mut lockfile = Lockfile::load().unwrap_or_default();

    let helm_client = RealHelmClient::new();
    let git_client = RealGitClient::new();
    let engine = SyncEngine::new(helm_client, git_client);

    let options = SyncOptions {
        ignore_skip: args.ignore_skip,
        only: args.only,
    };

    let stats = engine.sync(config, &mut lockfile, options, |event| match event {
        SyncEvent::ChartSkipped { name, reason } => {
            tracker.println(&format!(" {} {} ({})", style("[SKIP]").dim(), name, reason));
            tracker.inc();
        }
        SyncEvent::RepoUpdateStart => {
            tracker.set_message("Updating Helm repositories...");
        }
        SyncEvent::RepoUpdateSuccess => {}
        SyncEvent::RepoUpdateFailed(e) => {
            tracker.println(&format!(
                " {} Failed to update helm repos: {}",
                style("WARN:").yellow(),
                e
            ));
        }
        SyncEvent::ChartSyncStart { name } => {
            tracker.set_message(format!("Syncing {}...", name));
        }
        SyncEvent::ChartSyncSuccess { name, repo_type } => {
            tracker.println(&format!(
                " {}   {} {}",
                style("[OK]").green(),
                name,
                style(format!("({})", repo_type)).dim()
            ));
            tracker.inc();
        }
        SyncEvent::ChartSyncFailed { name, error } => {
            tracker.println(&format!(" {} {}: {}", style("[FAIL]").red(), name, error));
            tracker.inc();
        }
    })?;

    if stats.synced > 0
        && let Err(e) = lockfile.save()
    {
        tracker.println(&format!(
            "{} Failed to save lockfile: {}",
            style("WARN:").yellow(),
            e
        ));
    }

    tracker.finish_with_message("Sync completed");

    println!("\n\n{}", style("Summary:").bold());
    println!("  Synced:  {}", style(stats.synced).green());
    println!("  Failed:  {}", style(stats.failed).red());
    println!("  Skipped: {}", style(stats.skipped).yellow());

    if stats.failed > 0 {
        anyhow::bail!("Some charts failed to sync");
    }

    Ok(())
}
