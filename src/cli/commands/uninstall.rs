use crate::cli::commands::UninstallArgs;
use crate::clients::{HelmClient, helm::RealHelmClient};
use crate::config::Config;
use crate::config::error::ConfigError;
use crate::error::VesshelmError;
use crate::util::interaction::UserInteraction;
use crate::util::ui;
use console::style;

pub async fn run(
    args: UninstallArgs,
    config_path: &std::path::Path,
    interaction: &impl UserInteraction,
) -> Result<(), VesshelmError> {
    ui::print_step("Uninstalling chart...\n");

    // Load configuration
    let config = Config::load_from_path(config_path)?;

    // Select Chart
    let chart = if let Some(name) = args.name {
        config
            .charts
            .iter()
            .find(|c| c.name == name)
            .ok_or_else(|| ConfigError::ChartNotFound(name.clone()))?
    } else {
        // Interactive selection
        if config.charts.is_empty() {
            println!("No charts found in configuration.");
            return Ok(());
        }

        let mut sorted_charts: Vec<_> = config.charts.iter().collect();
        sorted_charts.sort_by(|a, b| a.name.cmp(&b.name));

        let items: Vec<String> = sorted_charts.iter().map(|c| c.selection_label()).collect();

        let selection = interaction.fuzzy_select("Select chart to uninstall", &items, 0)?;

        sorted_charts[selection]
    };

    // Check for dependents
    let dependents = crate::util::dag::get_dependents(&config.charts, &chart.name)?;

    if !dependents.is_empty() {
        println!(
            " {} The following charts depend on '{}':",
            style("WARN:").yellow().bright(),
            style(&chart.name).bold().bright()
        );
        for dep in &dependents {
            println!("  - {}", style(&dep.name).bold().bright());
        }
        println!();
    } else {
        println!(
            " {} No other charts depend on '{}'. It is safe to remove regarding dependencies.",
            style("[OK]").green().bright(),
            style(&chart.name).bold().bright()
        );
    }

    // Warning
    let release_name = chart.release_name();
    println!(
        " {} You are about to uninstall the release '{}' (chart '{}') from namespace '{}'.",
        style("WARN:").yellow().bright(),
        style(&release_name).bold().bright(),
        style(&chart.name).bold().bright(),
        style(&chart.namespace).bold().bright()
    );

    // Confirmation
    let confirmation = if args.no_interactive {
        true
    } else {
        interaction.confirm("Do you want to continue?", false)?
    };

    if !confirmation {
        println!(
            " {} Uninstallation aborted.",
            style("[ABORT]").dim().bright()
        );
        return Ok(());
    }

    println!("Uninstalling {}...", release_name);

    // Run helm uninstall
    let client = RealHelmClient::new();
    match client.uninstall(&release_name, &chart.namespace) {
        Ok(_) => {
            println!(
                " {} Successfully uninstalled release '{}'.",
                style("[OK]").green().bright(),
                style(&release_name).bold().bright()
            );
        }
        Err(e) => {
            eprintln!(
                " {} Failed to uninstall {}: {}",
                style("[FAIL]").red().bright(),
                release_name,
                e
            );
            return Err(crate::clients::error::ClientError::from(e).into());
        }
    }

    Ok(())
}
