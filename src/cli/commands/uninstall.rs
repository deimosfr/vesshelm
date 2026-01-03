use crate::cli::commands::UninstallArgs;
use anyhow::{Context, Result, anyhow};
// use colored::*; // Unused
use crate::clients::{HelmClient, helm::RealHelmClient};
use crate::config::Config;
use crate::util::interaction::UserInteraction;
use console::style;

pub async fn run(
    args: UninstallArgs,
    config_path: &std::path::Path,
    interaction: &impl UserInteraction,
) -> Result<()> {
    println!("{} Uninstalling chart...\n", style("==>").bold().green());

    // Load configuration
    let config = Config::load_from_path(config_path)?;

    // Select Chart
    let chart = if let Some(name) = args.name {
        config
            .charts
            .iter()
            .find(|c| c.name == name)
            .ok_or_else(|| anyhow!("Chart '{}' not found in vesshelm.yaml", name))?
    } else {
        // Interactive selection
        if config.charts.is_empty() {
            println!("No charts found in configuration.");
            return Ok(());
        }

        let mut sorted_charts: Vec<_> = config.charts.iter().collect();
        sorted_charts.sort_by(|a, b| a.name.cmp(&b.name));

        let items: Vec<String> = sorted_charts
            .iter()
            .map(|c| format!("{} ({})", c.name, c.namespace))
            .collect();

        let selection = interaction
            .fuzzy_select("Select chart to uninstall", &items, 0)
            .context("Failed to read selection")?;

        sorted_charts[selection]
    };

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
        interaction
            .confirm("Do you want to continue?", false)
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
