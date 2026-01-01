use super::DeleteArgs;
use crate::clients::{HelmClient, helm::RealHelmClient};
use crate::config::{Chart, Config};
use crate::lock::Lockfile;
use crate::util::config_updater::ConfigUpdater;
use crate::util::dag;
use crate::util::interaction::UserInteraction;
use anyhow::{Context, Result, anyhow};
use console::style;
use std::fs;
use std::path::Path;

pub async fn run(
    args: DeleteArgs,
    config_path: &Path,
    interaction: &impl UserInteraction,
) -> Result<()> {
    println!("{} Deleting charts...", style("==>").bold().green());

    // 1. Load configuration
    let config = Config::load_from_path(config_path)?;
    let mut lockfile = Lockfile::load()?;

    // 2. Select Chart
    let chart = if let Some(name) = args.name {
        // Check for matches
        let matches: Vec<&Chart> = config.charts.iter().filter(|c| c.name == name).collect();

        if matches.is_empty() {
            return Err(anyhow!("Chart '{}' not found in configuration", name));
        } else if matches.len() == 1 {
            matches[0]
        } else {
            // Ambiguous match
            println!(
                "Multiple charts found with name '{}'. Please select namespace:",
                name
            );
            let items: Vec<String> = matches
                .iter()
                .map(|c| format!("{} (ns: {})", c.name, c.namespace))
                .collect();

            let selection = interaction
                .fuzzy_select("Select chart instance", &items, 0)
                .context("Failed to read selection")?;

            matches[selection]
        }
    } else {
        // Interactive selection from all charts
        if config.charts.is_empty() {
            println!("No charts found in configuration.");
            return Ok(());
        }

        // Create a sorted list of references for selection index mapping
        let mut sorted_charts: Vec<_> = config.charts.iter().collect();
        sorted_charts.sort_by(|a, b| a.name.cmp(&b.name));

        let items: Vec<String> = sorted_charts
            .iter()
            .map(|c| format!("{} ({})", c.name, style(&c.namespace).dim()))
            .collect();

        let selection = interaction
            .fuzzy_select("Select chart to delete", &items, 0)
            .context("Failed to read selection")?;

        sorted_charts[selection]
    };

    let chart_name = &chart.name;
    let chart_ns = &chart.namespace;

    // 3. Check Dependencies
    let dependents = dag::get_dependents(&config.charts, chart_name)?;
    if !dependents.is_empty() {
        println!(
            "{} The following charts depend on '{}':",
            style("⚠️ ").yellow(),
            style(chart_name).bold()
        );
        for dep in dependents {
            println!("  - {}", style(&dep.name).bold());
        }
        println!("Cannot delete chart while it has dependents.");
        return Ok(());
    }

    // 4. Calculate Impact / Summary
    let chart_path = resolve_delete_path(&config, chart)?;
    let path_exists = chart_path.exists();

    let repo_to_remove = if let Some(repo_name) = &chart.repo_name {
        // Check if this repo is used by any OTHER chart
        // We must exclude the chart we are deleting from the check
        let is_used_by_others = config.charts.iter().any(|c| {
            // Check if it's a DIFFERENT chart instance
            // (Same name AND same namespace = SAME chart)
            let is_same_chart = &c.name == chart_name && &c.namespace == chart_ns;
            !is_same_chart && c.repo_name.as_deref() == Some(repo_name)
        });

        if !is_used_by_others {
            Some(repo_name.clone())
        } else {
            None
        }
    } else {
        None
    };

    // Ask for uninstall if interactive
    let uninstall_release = if !args.no_interactive {
        interaction
            .confirm("Do you also want to uninstall the Helm release?", false)
            .unwrap_or(false)
    } else {
        false
    };

    println!("\n{}", style("Deletion Summary:").bold());
    println!("{:<25} {}", "  Chart:", style(chart_name).bold().red());
    println!("{:<25} {}", "  Namespace:", style(chart_ns).bold());
    println!(
        "{:<25} {}",
        "  Directory:",
        style(chart_path.display()).bold()
    );

    let status_str = if path_exists {
        style("Present (will be deleted)").green()
    } else {
        style("Missing (already deleted)").yellow()
    };
    println!("{:<25} {}", "  Status:", status_str);

    if let Some(repo) = &repo_to_remove {
        println!(
            "{:<25} {} (no longer used)",
            "  Repository:",
            style(repo).bold()
        );
    }

    let uninstall_msg = if uninstall_release {
        style("Yes").red().bold()
    } else {
        style("No").dim()
    };
    println!("{:<25} {}", "  Uninstall release:", uninstall_msg);

    let mut action_parts = Vec::new();
    if uninstall_release {
        action_parts.push("Uninstall Helm release");
    }
    if path_exists {
        action_parts.push("delete local directory");
    }
    action_parts.push("remove from vesshelm.yaml and vesshelm.lock");

    println!("{:<25} {}", "  Action:", action_parts.join(", "));

    // 5. Confirmation
    if !args.no_interactive && !interaction.confirm("Do you want to continue?", false)? {
        println!(" {} Deletion aborted.", style("[ABORT]").dim());
        return Ok(());
    }

    // 6. Execution
    println!();

    // 6.0 Uninstall Release (Critical Step)
    if uninstall_release {
        println!(
            "{} Uninstalling Helm release...",
            style("==>").bold().green()
        );
        let client = RealHelmClient::new();
        match client.uninstall(chart_name, chart_ns) {
            Ok(_) => println!(
                " {} Release uninstalled (or not found).",
                style("[OK]").green()
            ),
            Err(e) => {
                eprintln!(
                    " {} Failed to uninstall release: {}",
                    style("[FAIL]").red(),
                    e
                );
                eprintln!("Aborting deletion to preserve configuration.");
                return Ok(());
            }
        }
    }

    // 6.1 Delete Directory
    if path_exists {
        println!(
            "{} Cleaning configs and removing directory...",
            style("==>").bold().green()
        );
        fs::remove_dir_all(&chart_path)
            .with_context(|| format!("Failed to delete directory {}", chart_path.display()))?;
        println!(" {} Directory removed.", style("[OK]").green());
    } else {
        println!(
            "Directory {} does not exist, skipping.",
            chart_path.display()
        );
    }

    // 6.2 Update Lockfile
    if let Some(repo_name) = &chart.repo_name {
        lockfile.remove(chart_name, repo_name);
        lockfile.save().context("Failed to save lockfile")?;
        println!(" {} Removed from lockfile.", style("[OK]").green());
    }

    // 6.3 Update Config (Remove Chart non-destructively)
    ConfigUpdater::remove_chart(config_path, chart_name, chart_ns)?;
    println!(" {} Removed chart from config.", style("[OK]").green());

    // 6.4 Update Config (Remove Repo if needed non-destructively)
    if let Some(repo_name) = repo_to_remove {
        ConfigUpdater::remove_repository(config_path, &repo_name)?;
        println!(
            " {} Removed unused repository '{}'.",
            style("[OK]").green(),
            repo_name
        );
    }

    println!(" {} Deletion completed.", style("[OK]").green());

    Ok(())
}

fn resolve_delete_path(config: &Config, chart: &Chart) -> Result<std::path::PathBuf> {
    use std::path::PathBuf;

    // 1. Local Chart Priority
    if chart.repo_name.is_none()
        && let Some(path) = &chart.chart_path
    {
        return Ok(PathBuf::from(path));
    }

    let resolved_base = config.resolve_chart_destination(chart)?;
    let standard_path = resolved_base.join(&chart.name);

    // 2. Explicit Destination Adaptation
    if let Some(dest_str) = &chart.dest {
        // Check if it is a known destination alias
        let is_alias = config.destinations.iter().any(|d| &d.name == dest_str);

        if !is_alias {
            // It's an explicit path override.
            // Heuristic: If standard path doesn't exist, but the base path does,
            // assume the user pointed directly to the chart folder.
            if !standard_path.exists() && resolved_base.exists() {
                return Ok(resolved_base);
            }
        }
    }

    Ok(standard_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Destination;

    #[test]
    fn test_resolve_delete_path_standard() {
        let config = Config {
            repositories: vec![],
            charts: vec![],
            destinations: vec![Destination {
                name: "default".to_string(),
                path: "charts".to_string(),
            }],
            helm: None,
        };
        let chart = Chart {
            name: "nginx".to_string(),
            repo_name: Some("stable".to_string()),
            version: Some("1.0".to_string()),
            namespace: "default".to_string(),
            dest: None, // Uses default
            chart_path: None,
            no_sync: false,
            no_deploy: false,
            comment: None,
            values_files: None,
            helm_args_append: None,
            helm_args_override: None,
            values: None,
            depends: None,
        };

        let path = resolve_delete_path(&config, &chart).unwrap();
        assert_eq!(path.to_string_lossy(), "charts/nginx");
    }

    #[test]
    fn test_resolve_delete_path_local_chart() {
        let config = Config {
            repositories: vec![],
            charts: vec![],
            destinations: vec![Destination {
                name: "default".to_string(),
                path: "charts".to_string(),
            }],
            helm: None,
        };
        let chart = Chart {
            name: "my-local".to_string(),
            repo_name: None,
            version: None,
            namespace: "default".to_string(),
            dest: None,
            chart_path: Some("custom/path/my-local".to_string()),
            no_sync: false,
            no_deploy: false,
            comment: None,
            values_files: None,
            helm_args_append: None,
            helm_args_override: None,
            values: None,
            depends: None,
        };

        let path = resolve_delete_path(&config, &chart).unwrap();
        assert_eq!(path.to_string_lossy(), "custom/path/my-local");
    }

    #[test]
    fn test_resolve_delete_path_explicit_dest_adaptation() {
        // This test requires file system interaction simulation or mocking.
        // For unit test simplicity in this environment, we verify default fallback
        // when files don't exist.

        let config = Config {
            repositories: vec![],
            charts: vec![],
            destinations: vec![Destination {
                name: "default".to_string(),
                path: "charts".to_string(),
            }],
            helm: None,
        };
        let chart = Chart {
            name: "nginx".to_string(),
            repo_name: Some("stable".to_string()),
            version: Some("1.0".to_string()),
            namespace: "default".to_string(),
            dest: Some("custom/folder".to_string()),
            chart_path: None,
            no_sync: false,
            no_deploy: false,
            comment: None,
            values_files: None,
            helm_args_append: None,
            helm_args_override: None,
            values: None,
            depends: None,
        };

        // Default behavior (fallback to join)
        let path = resolve_delete_path(&config, &chart).unwrap();
        assert_eq!(path.to_string_lossy(), "custom/folder/nginx");
    }
}
