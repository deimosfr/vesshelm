use anyhow::{Context, Result, anyhow};
use colored::*;
use console::style;
use dialoguer::Confirm;
use std::process::Command;
use vesshelm::config::Config;

use super::UninstallArgs;

pub async fn run(args: UninstallArgs, config_path: &std::path::Path) -> Result<()> {
    let name = args.name;
    println!("{} Uninstalling {}...", style("==>").bold().green(), name);

    // Load configuration
    let config = Config::load_from_path(config_path)?;

    // Find chart
    let chart = config
        .charts
        .iter()
        .find(|c| c.name == name)
        .ok_or_else(|| anyhow!("Chart '{}' not found in vesshelm.yaml", name))?;

    // Check for dependents
    let dependents = vesshelm::util::dag::get_dependents(&config.charts, &chart.name)
        .context("Failed to check chart dependencies")?;

    if !dependents.is_empty() {
        println!(
            "{} The following charts depend on '{}':",
            "⚠️ ".yellow(),
            style(&chart.name).bold()
        );
        for dep in &dependents {
            println!("  - {}", style(&dep.name).bold());
        }
        println!();
    } else {
        println!(
            "{} No other charts depend on '{}'. It is safe to remove regarding dependencies.",
            "✅ ".green(),
            style(&chart.name).bold()
        );
    }

    // Warning
    println!(
        "{} You are about to uninstall the chart '{}' from namespace '{}'.",
        "⚠️ ".yellow(),
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
        println!("{} Uninstallation aborted.", "🚫 ".dimmed());
        return Ok(());
    }

    println!("{} Uninstalling {}...", "🗑️ ".red(), chart.name);

    // Run helm uninstall
    // helm uninstall <name> -n <namespace>
    let output = Command::new("helm")
        .arg("uninstall")
        .arg(&chart.name)
        .arg("-n")
        .arg(&chart.namespace)
        .output()
        .context("Failed to execute helm uninstall command")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!(
            "{} Failed to uninstall {}: {}",
            "❌ ".red(),
            chart.name,
            stderr
        );
        return Err(anyhow!("Helm uninstall failed"));
    }

    println!(
        "{} Successfully uninstalled {}.",
        "✅ ".green(),
        style(&chart.name).bold()
    );

    Ok(())
}
