use crate::config::{Config, Repository};
use crate::util::config_updater::{ChartConfig, ConfigUpdater};
use crate::util::interaction::UserInteraction;
use anyhow::{Context, Result};
use console::style;
use std::collections::HashSet;
use std::path::Path;

mod source;
use source::get_source;

pub async fn run(config_path: &Path, interaction: &(impl UserInteraction + Sync)) -> Result<()> {
    println!("{} Adding chart...", style("==>").bold().green());

    // 1. Source Selection
    let sources = vec![
        "Artifact Hub".to_string(),
        "Git".to_string(),
        "OCI".to_string(),
    ];
    let selection = interaction
        .select("Select source type", &sources, 0)
        .context("Failed to read source selection")?;

    let source = get_source(selection).context("Invalid source selection")?;

    // 2. Get Details from Source
    let details = source.prompt_details(interaction).await?;

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

        // Validation Loop for Repo Name
        let name = loop {
            let n = interaction.input("New Repository Name", Some(&default_name))?;
            if config.repositories.iter().any(|r| r.name == n) {
                println!("Repository name already exists. Please choose another.");
                continue;
            }
            break n;
        };

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
    let chart_name = interaction.input("Chart Name", Some(&default_chart_name))?;

    // Namespace Selection
    let mut namespaces: Vec<String> = config
        .charts
        .iter()
        .map(|c| c.namespace.clone())
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();
    namespaces.sort();

    let namespace = if namespaces.is_empty() {
        interaction.input("Namespace", None)?
    } else {
        let mut selection_items = namespaces.clone();
        selection_items.push("Create new...".to_string());

        let selection = interaction.select("Select Namespace", &selection_items, 0)?;

        if selection == namespaces.len() {
            interaction.input("New Namespace", None)?
        } else {
            namespaces[selection].clone()
        }
    };

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
        if !interaction.confirm("Continue anyway (will duplicate)?", false)? {
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

    if interaction.confirm("Add to config?", true)? {
        let chart_config = ChartConfig {
            name: chart_name.clone(),
            repo_name: repo_name_to_use,
            namespace,
            version: details.version,
            chart_path: details.chart_path,
            comment: details.comment,
        };

        ConfigUpdater::update(config_path, new_repo, chart_config)?;
        println!("vesshelm.yaml updated.");

        if interaction.confirm("Run sync now?", true)? {
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
