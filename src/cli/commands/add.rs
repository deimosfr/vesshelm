use anyhow::{Context, Result};
use console::style;
use dialoguer::{Confirm, Input, theme::ColorfulTheme};
use regex::Regex;
use std::path::Path;
use vesshelm::clients::artifacthub::{ArtifactHubClient, Package};
use vesshelm::config::{Config, RepoType, Repository};

pub async fn run(config_path: &Path) -> Result<()> {
    // 1. Prompt for Artifact Hub URL
    let url: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Enter Artifact Hub URL")
        .interact_text()
        .context("Failed to read URL input")?;

    // 2. Parse URL
    let re = Regex::new(r"artifacthub\.io/packages/helm/([^/]+)/([^/?]+)")
        .context("Failed to compile regex")?;

    let caps = re.captures(&url).ok_or_else(|| anyhow::anyhow!("Invalid Artifact Hub URL format. Expected: https://artifacthub.io/packages/helm/<repo>/<chart>"))?;
    let repo_name = caps.get(1).map_or("", |m| m.as_str());
    let chart_name = caps.get(2).map_or("", |m| m.as_str());

    println!("Detected: Repo={}, Chart={}", repo_name, chart_name);

    // 3. Fetch details
    let client = ArtifactHubClient::new();
    let package: Package = client
        .get_package_details(repo_name, chart_name)
        .await
        .context("Failed to fetch package details")?;

    println!(
        "Found chart: {} v{} from repo {}",
        package.name, package.version, package.repository.url
    );

    // 4. Configuration Logic
    let config = Config::load_from_path(config_path)?;

    // Check Repo
    let repo_url = &package.repository.url;

    // Try to find existing repo by URL (ignoring trailing slash)
    let existing_repo = config
        .repositories
        .iter()
        .find(|r| r.url.trim_end_matches('/') == repo_url.trim_end_matches('/'));

    let (repo_name_to_use, new_repo) = if let Some(r) = existing_repo {
        println!("Repository URL {} already exists as '{}'", repo_url, r.name);
        (r.name.clone(), None)
    } else {
        println!("Repository {} not found in config.", repo_url);
        // Default name: try Artifact Hub repo name first
        let default_name = package.repository.name.clone();

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
                r#type: RepoType::Helm,
            }),
        )
    };

    // Chart details
    // The chart name from Artifact Hub package might be "vaultwarden" but from URL "gissilabs/vaultwarden"
    // We already parsed `chart_name` from URL which corresponds to "vaultwarden" in "helm/gissilabs/vaultwarden"
    // Wait, regex was: artifacthub.io/packages/helm/([^/]+)/([^/?]+)
    // 1: repo (gissilabs), 2: chart (vaultwarden)
    // Package details name usually matches the chart name.

    let default_chart_name = package.name.clone();
    let chart_name: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Chart Name")
        .default(default_chart_name)
        .validate_with(|_input: &String| -> Result<(), &str> { Ok(()) })
        .interact_text()?;

    let namespace: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Namespace")
        .interact_text()?;

    // Check uniqueness (warning only, as per previous logic, or strict?)
    // keeping warning logic
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

    // Prepare data for summary and appending
    // NOTE: We do NOT create a full Chart struct for serialization anymore,
    // but we use one transiently if needed for display?
    // Actually we just need the values.

    // Summary
    println!("\n\n{}", style("Summary:").bold());
    if let Some(r) = &new_repo {
        println!(
            " {}  Repository: {} {}",
            style("[NEW]").green(),
            style(&r.name).bold(),
            style(format!("({})", r.url)).dim()
        );
    } else {
        println!(
            " {}   Repository: {}",
            style("[OK]").blue(),
            style(&repo_name_to_use).bold()
        );
    }
    println!(
        " {}  Chart: {} {}",
        style("[NEW]").green(),
        style(&chart_name).bold(),
        style(format!("(v{})", package.version)).dim()
    );
    println!("        Namespace: {}", namespace);
    println!("        Repo: {}", repo_name_to_use);

    if Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("Add to config?")
        .default(true)
        .interact()?
    {
        // MANUAL UPDATE LOGIC
        let mut file_content = std::fs::read_to_string(config_path)?;

        if let Some(r) = new_repo {
            // Append to repositories section
            if !file_content.contains("repositories:") {
                file_content.push_str("\nrepositories:\n");
            }
            // Simple heuristic to find end of repositories or append to end of file if it's the last section?
            // Safer: Just append to the repositories section text block?
            // Or just append valid YAML list item?
            // "Smart append":
            // 1. Find "repositories:"
            // 2. Find the end of properties of the last item? Hard without parser.
            // Simplified: If "repositories:" exists, append to file end? No, order matters for structure?
            // YAML order of sections doesn't matter. But items should be under the section.
            // If we just append "  - name: ...\n    url: ..." under "repositories:", it works.

            // Let's assume standard format where sections are root level.
            // We'll append entries to the appropriate sections by finding the last occurrence of a list item in that section,
            // OR just appending to the section header if empty?

            // Robust enough approach for this tool:
            // Append new repo to the `repositories` list in the string.
            // We iterate lines to find `repositories:`.
            // Then we look for the next root level key (no indentation) to find the end of block,
            // NOT perfect but decent.

            // Strategy:
            // Find "repositories:" line.
            // Insert after it? Or at the end of the block?
            // Inserting after "repositories:" is safe for list.

            let repo_block = format!("\n  - name: {}\n    url: {}", r.name, r.url);

            if let Some(idx) = file_content.find("repositories:") {
                let insert_at = idx + "repositories:".len();
                file_content.insert_str(insert_at, &repo_block);
            } else {
                // Add section if missing (unlikely based on valid config load)
                file_content.push_str("\nrepositories:");
                file_content.push_str(&repo_block);
            }
        }

        // Add Chart
        let chart_block_fixed = format!(
            "\n  - name: {}\n    repo_name: {}\n    namespace: {}\n    version: {}\n    comment: {}",
            chart_name, repo_name_to_use, namespace, package.version, url
        );

        if let Some(idx) = file_content.find("charts:") {
            let insert_at = idx + "charts:".len();
            file_content.insert_str(insert_at, &chart_block_fixed);
        } else {
            file_content.push_str("\ncharts:");
            file_content.push_str(&chart_block_fixed);
        }

        std::fs::write(config_path, file_content)?;
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
