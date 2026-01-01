use anyhow::{Context, Result};
use console::style;
use dialoguer::{Confirm, Input, Select, theme::ColorfulTheme};
use std::path::Path;
use vesshelm::config::{Config, Repository};

mod source;

use source::get_source;
use vesshelm::util::config_updater::{ChartConfig, ConfigUpdater};

pub async fn run(config_path: &Path) -> Result<()> {
    println!("{} Adding chart...", style("==>").bold().green());

    // 1. Source Selection
    let sources = vec!["Artifact Hub", "Git", "OCI"];
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select source type")
        .default(0)
        .items(&sources)
        .interact()
        .context("Failed to read source selection")?;

    let source = get_source(selection).context("Invalid source selection")?;

    // 2. Get Details from Source
    let details = source.prompt_details().await?;

    // 3. Configuration Logic
    let config = Config::load_from_path(config_path)?;

    // Check Repo
    let repo_url = &details.repo_url;
    let existing_repo = config
        .repositories
        .iter()
        .find(|r| r.url.trim_end_matches('/') == repo_url.trim_end_matches('/'));

    let (repo_name_to_use, new_repo) = if let Some(r) = existing_repo {
        println!("Repository URL {} already exists as '{}'", repo_url, r.name);
        (r.name.clone(), None)
    } else {
        println!("Repository {} not found in config.", repo_url);
        let default_name = details.repo_name.clone();

        let name: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("New Repository Name")
            .default(default_name)
            .validate_with(|input: &String| -> Result<(), &str> {
                if config.repositories.iter().any(|r| r.name == *input) {
                    Err("Repository name already exists")
                } else {
                    Ok(())
                }
            })
            .interact_text()?;

        (
            name.clone(),
            Some(Repository {
                name,
                url: repo_url.clone(),
                r#type: details.repo_type,
            }),
        )
    };

    // Chart Details
    let default_chart_name = details.chart_name.clone();
    let chart_name: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Chart Name")
        .default(default_chart_name)
        .validate_with(|_input: &String| -> Result<(), &str> { Ok(()) })
        .interact_text()?;

    let namespace: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Namespace")
        .interact_text()?;

    // Check uniqueness
    if config
        .charts
        .iter()
        .any(|c| c.name == chart_name && c.namespace == namespace)
    {
        println!(
            "{} Chart with name '{}' and namespace '{}' already exists!",
            style("WARN:").yellow(),
            chart_name,
            namespace
        );
        if !Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("Continue anyway (will duplicate)?")
            .default(false)
            .interact()?
        {
            return Ok(());
        }
    }

    // Summary
    println!("\n\n{}", style("Summary:").bold());
    if let Some(r) = &new_repo {
        println!(
            " {:<7} Repository: {} {}",
            style("[NEW]").green(),
            style(&r.name).bold(),
            style(format!("({})", r.url)).dim()
        );
    } else {
        println!(
            " {:<7} Repository: {}",
            style("[OK]").green(),
            style(&repo_name_to_use).bold()
        );
    }

    let version_display = details.version.clone().unwrap_or_default();
    println!(
        " {:<7} Chart: {}",
        style("[NEW]").green(),
        style(&chart_name).bold()
    );
    println!("         Version: {}", version_display);
    println!("         Namespace: {}", namespace);

    if Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("Add to config?")
        .default(true)
        .interact()?
    {
        let chart_config = ChartConfig {
            name: chart_name.clone(),
            repo_name: repo_name_to_use,
            namespace,
            version: details.version,
            chart_path: details.chart_path,
            comment: None, // We decided to omit comment for now unless specific requirement comes up
        };

        ConfigUpdater::update(config_path, new_repo, chart_config)?;
        println!("vesshelm.yaml updated.");

        if Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("Run sync now?")
            .default(true)
            .interact()?
        {
            use crate::cli::commands::SyncArgs;
            let args = SyncArgs {
                charts: Some(vec![chart_name]),
                ignore_skip: false,
            };
            crate::cli::commands::sync::run(args, false, config_path).await?;
        }
    } else {
        println!("Aborted. No changes made.");
    }

    Ok(())
}
