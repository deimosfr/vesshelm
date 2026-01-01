use anyhow::{Context, Result, anyhow};
// use colored::*; // Unused
use crate::clients::{HelmClient, helm::RealHelmClient};
use crate::config::Config;
use console::style;
use dialoguer::Confirm;

use super::UninstallArgs;

pub async fn run(args: UninstallArgs, config_path: &std::path::Path) -> Result<()> {
    let name = args.name;
    println!("Uninstalling {}...", name);

    // Load configuration
    let config = Config::load_from_path(config_path)?;

    // Find chart
    let chart = config
        .charts
        .iter()
        .find(|c| c.name == name)
        .ok_or_else(|| anyhow!("Chart '{}' not found in vesshelm.yaml", name))?;

    // Check for dependents
    let dependents = crate::util::dag::get_dependents(&config.charts, &chart.name)
        .context("Failed to check chart dependencies")?;

    if !dependents.is_empty() {
        println!(
            " {} The following charts depend on '{}':",
            style("WARN:").yellow(),
            style(&chart.name).bold()
        );
        for dep in &dependents {
            println!("  - {}", style(&dep.name).bold());
        }
        println!();
    } else {
        println!(
            " {} No other charts depend on '{}'. It is safe to remove regarding dependencies.",
            style("[OK]").green(),
            style(&chart.name).bold()
        );
    }

    // Warning
    println!(
        " {} You are about to uninstall the chart '{}' from namespace '{}'.",
        style("WARN:").yellow(),
        style(&chart.name).bold(),
        style(&chart.namespace).bold()
    );

    // Confirmation
    let confirmation = if args.no_interactive {
        true
    } else {
        Confirm::new()
            .with_prompt("Do you want to continue?")
            .default(false)
            .interact()
            .context("Failed to read user confirmation")?
    };

    if !confirmation {
        println!(" {} Uninstallation aborted.", style("[ABORT]").dim());
        return Ok(());
    }

    println!("Uninstalling {}...", chart.name);

    // Run helm uninstall
    let client = RealHelmClient::new();
    match client.uninstall(&chart.name, &chart.namespace) {
        Ok(_) => {
            println!(
                " {} Successfully uninstalled {}.",
                style("[OK]").green(),
                style(&chart.name).bold()
            );
        }
        Err(e) => {
            eprintln!(
                " {} Failed to uninstall {}: {}",
                style("[FAIL]").red(),
                chart.name,
                e
            );
            return Err(e);
        }
    }

    Ok(())
}
