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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util::interaction::UserInteraction;
    use anyhow::Result;
    use std::sync::{Arc, Mutex};
    use tempfile::NamedTempFile;

    // Manual Mock because mockall might be heavy to setup inline or requires traits to be visible
    #[derive(Default, Clone)]
    struct MockInteraction {
        inputs: Arc<Mutex<Vec<String>>>,
        selects: Arc<Mutex<Vec<usize>>>,
        confirms: Arc<Mutex<Vec<bool>>>,
    }

    impl MockInteraction {
        fn new() -> Self {
            Self::default()
        }

        fn push_input(&self, val: &str) {
            self.inputs.lock().unwrap().push(val.to_string());
        }

        fn push_select(&self, val: usize) {
            self.selects.lock().unwrap().push(val);
        }

        fn push_confirm(&self, val: bool) {
            self.confirms.lock().unwrap().push(val);
        }
    }

    impl UserInteraction for MockInteraction {
        fn confirm(&self, _prompt: &str, _default: bool) -> Result<bool> {
            let mut confirms = self.confirms.lock().unwrap();
            if !confirms.is_empty() {
                Ok(confirms.remove(0))
            } else {
                Ok(true) // Default
            }
        }

        fn input(&self, _prompt: &str, _default: Option<&str>) -> Result<String> {
            let mut inputs = self.inputs.lock().unwrap();
            if !inputs.is_empty() {
                Ok(inputs.remove(0))
            } else {
                Ok("default_input".to_string())
            }
        }

        fn select(&self, _prompt: &str, _items: &[String], _default: usize) -> Result<usize> {
            let mut selects = self.selects.lock().unwrap();
            if !selects.is_empty() {
                Ok(selects.remove(0))
            } else {
                Ok(0)
            }
        }

        fn fuzzy_select(&self, _prompt: &str, _items: &[String], _default: usize) -> Result<usize> {
            Ok(0)
        }
    }

    #[tokio::test]
    async fn test_add_command_git_flow_new_repo() -> Result<()> {
        let interaction = MockInteraction::new();

        // 1. Source Selection: "Git" is index 1
        interaction.push_select(1);

        // 2. Git Details Inputs (FIFO order of popping, so push in REVERSE or implement queue correctly)
        // Implementation uses remove(0), so we push in ORDER.

        // GitSource inputs:
        interaction.push_input("https://github.com/org/repo.git"); // URL
        interaction.push_input("charts/my-chart"); // Path
        interaction.push_input("v1.0.0"); // Version

        // 3. Config Logic
        // "Repository ... not found" -> ask for name
        interaction.push_input("my-git-repo"); // New Repository Name

        // Chart Name
        interaction.push_input("my-chart");

        // Namespace Selection
        // Assuming empty config, it asks for input directly
        interaction.push_input("my-ns");

        // Summary Confirmations
        interaction.push_confirm(true); // Add to config?
        interaction.push_confirm(false); // Run sync now? (Skip to avoid running actual sync commands)

        // Setup Config
        let config_file = NamedTempFile::new()?;
        let initial_config = r#"
repositories:
charts:
destinations:
"#;
        std::fs::write(&config_file, initial_config)?;

        run(config_file.path(), &interaction).await?;

        // Verify content
        let config = Config::load_from_path(config_file.path())?;

        assert_eq!(config.repositories.len(), 1);
        assert_eq!(config.repositories[0].name, "my-git-repo");
        assert_eq!(
            config.repositories[0].url,
            "https://github.com/org/repo.git"
        );

        assert_eq!(config.charts.len(), 1);
        assert_eq!(config.charts[0].name, "my-chart");
        assert_eq!(config.charts[0].repo_name, Some("my-git-repo".to_string()));
        assert_eq!(config.charts[0].version, Some("v1.0.0".to_string()));

        Ok(())
    }

    #[tokio::test]
    async fn test_add_command_collision_check() -> Result<()> {
        let interaction = MockInteraction::new();

        // 1. Source Selection: "Git"
        interaction.push_select(1);

        // 2. Git inputs
        interaction.push_input("https://github.com/org/repo.git");
        interaction.push_input("charts/collision");
        interaction.push_input("v2.0.0");

        // 3. Config Logic
        // Repo exists check logic.
        // If we use the same URL, it matches the existing repo.
        // But our inputs above set URL to "https://github.com/org/repo.git".
        // The existing repo "repo-collision" below uses that URL.
        // So prompt finding existing repo happens!
        // "Repository URL ... already exists as 'repo-collision'"
        // It does NOT ask for repo name.

        // Chart Name
        interaction.push_input("collision-chart");

        // Namespace
        interaction.push_input("default");

        // COLLISION CHECK:
        // We will prep config to already have this chart.
        // It should ask "Continue duplicated?"
        interaction.push_confirm(true);

        // Add to config?
        interaction.push_confirm(true);
        // Sync?
        interaction.push_confirm(false);

        // Setup Pre-existing Config
        let config_file = NamedTempFile::new()?;
        let initial_config = r#"
repositories:
  - name: repo-collision
    url: https://github.com/org/repo.git
    type: git
charts:
  - name: collision-chart
    namespace: default
    repo_name: repo-collision
    version: v1.0.0
destinations:
"#;
        std::fs::write(&config_file, initial_config)?;

        run(config_file.path(), &interaction).await?;

        // Verify we added a SECOND chart

        // Use raw load because Config::load_from_path validates, and we intentionally have a duplicate
        let content = std::fs::read_to_string(config_file.path())?;
        let config: Config = serde_yaml_ng::from_str(&content)?;
        // We expect 2 charts now.
        assert_eq!(config.charts.len(), 2);
        // ConfigUpdater prepends new charts, so the new one is at index 0
        assert_eq!(config.charts[0].version, Some("v2.0.0".to_string()));
        assert_eq!(config.charts[1].version, Some("v1.0.0".to_string()));

        Ok(())
    }
}
