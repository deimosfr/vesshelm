use anyhow::{Context, Result, anyhow};
use colored::*;
use console::style;
use semver::Version;
use vesshelm::config::{Config, RepoType};

use std::process::Command;

use super::sync;
use super::{CheckUpdatesArgs, SyncArgs};

pub async fn run(
    args: CheckUpdatesArgs,
    no_progress: bool,
    config_path: &std::path::Path,
) -> Result<()> {
    // Load configuration
    let mut config = Config::load_from_path(config_path)?;

    // Update Helm repositories
    update_helm_repos()?;

    let mut updates_found = false;
    let mut charts_to_update = Vec::new();

    println!(
        "{} 🔍 Calculating dependency graph",
        style("==>").bold().green()
    );

    // Iterate over charts
    for chart in &mut config.charts {
        // Filter by positional charts arg
        if let Some(charts) = &args.charts
            && !charts.contains(&chart.name)
        {
            continue;
        }

        print!("checking {}... ", chart.name);

        // Skip non-Helm charts
        let repo_type = if let Some(repo_name) = &chart.repo_name {
            config
                .repositories
                .iter()
                .find(|r| r.name == *repo_name)
                .map(|r| &r.r#type)
        } else {
            None
        };

        match repo_type {
            Some(RepoType::Helm) => {
                // Determine chart reference: repo/chart
                let chart_ref = if let Some(repo_name) = &chart.repo_name {
                    format!("{}/{}", repo_name, chart.name)
                } else {
                    chart.name.clone()
                };

                match get_latest_version(&chart_ref) {
                    Ok(latest_version_str) => {
                        let current_version_str = chart.version.as_deref().unwrap_or("unknown");

                        // Parse versions using semver
                        let current_semver = parse_version(current_version_str);
                        let latest_semver = parse_version(&latest_version_str);

                        match (current_semver, latest_semver) {
                            (Ok(current), Ok(latest)) => match latest.cmp(&current) {
                                std::cmp::Ordering::Greater => {
                                    println!(
                                        "{} {} -> {}",
                                        "Outdated".yellow(),
                                        current_version_str.dimmed(),
                                        latest_version_str.green()
                                    );
                                    updates_found = true;
                                    charts_to_update.push((chart.name.clone(), latest_version_str));
                                }
                                _ => println!("{}", "Up to date".green()),
                            },
                            _ => {
                                // Fallback to string comparison if semantics parsing fails
                                if latest_version_str != current_version_str {
                                    println!(
                                        "{} {} -> {} (semver parse failed)",
                                        "Outdated".yellow(),
                                        current_version_str.dimmed(),
                                        latest_version_str.green()
                                    );
                                    updates_found = true;
                                    charts_to_update.push((chart.name.clone(), latest_version_str));
                                } else {
                                    println!("{}", "Up to date".green());
                                }
                            }
                        }
                    }
                    Err(e) => {
                        println!("{} Failed to fetch version: {}", "Error".red(), e);
                    }
                }
            }
            _ => {
                println!("{}", "Skipped (local/git/oci)".dimmed());
            }
        }
    }

    let should_apply = args.apply || args.apply_sync;

    use vesshelm::util::config_updater::ConfigUpdater;

    // ... existing imports ...

    match (should_apply, updates_found) {
        (true, true) => {
            println!("\n{} Applying updates...", "📝".bold());

            // Apply updates
            for (chart_name, new_version) in &charts_to_update {
                if let Err(e) =
                    ConfigUpdater::update_chart_version(config_path, chart_name, new_version)
                {
                    println!(
                        "{} Failed to update {}: {}",
                        "Warning".yellow(),
                        chart_name,
                        e
                    );
                } else {
                    println!("Updated {} to {}", chart_name.bold(), new_version.green());
                }
            }

            println!("{} vesshelm.yaml updated.", "✅".green());

            if args.apply_sync {
                println!();
                let sync_args = SyncArgs {
                    charts: args.charts,
                    ignore_skip: false,
                };
                sync::run(sync_args, no_progress, config_path).await?;
            }
        }
        (false, true) => {
            println!("\nRun with {} to apply changes.", "--apply".cyan());
        }
        _ => {
            println!("\nAll checked charts are up to date.");
        }
    }

    Ok(())
}

fn update_helm_repos() -> Result<()> {
    println!("{} Updating Helm repositories...", "🔄".dimmed());
    let status = Command::new("helm")
        .arg("repo")
        .arg("update")
        .status()
        .context("Failed to execute helm repo update")?;

    if !status.success() {
        return Err(anyhow!("Failed to update helm repositories"));
    }
    Ok(())
}

fn get_latest_version(chart_ref: &str) -> Result<String> {
    let output = Command::new("helm")
        .arg("search")
        .arg("repo")
        .arg(chart_ref)
        .arg("--output")
        .arg("yaml")
        .output()
        .context("Failed to execute helm search")?;

    if !output.status.success() {
        return Err(anyhow!("helm search failed"));
    }
    // Parse YAML output
    let versions: Vec<serde_yaml_ng::Value> =
        serde_yaml_ng::from_slice(&output.stdout).context("Failed to parse helm search output")?;

    if let Some(chart_value) = versions.first() {
        if let Some(version_str) = chart_value["version"].as_str() {
            Ok(version_str.to_string())
        } else {
            Err(anyhow!(
                "'version' field not found or not a string in chart output"
            ))
        }
    } else {
        Err(anyhow!("Chart not found"))
    }
}

fn parse_version(v: &str) -> Result<Version, semver::Error> {
    Version::parse(v.strip_prefix('v').unwrap_or(v))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_version() {
        assert_eq!(parse_version("1.2.3").unwrap(), Version::new(1, 2, 3));
        assert_eq!(parse_version("v1.2.3").unwrap(), Version::new(1, 2, 3));
        assert!(parse_version("invalid").is_err());
    }

    #[test]
    fn test_version_comparison() {
        let v1 = parse_version("1.18.4").unwrap();
        let v2 = parse_version("1.19.0-pre.3").unwrap();
        // 1.19.0-pre.3 is NEWER than 1.18.4.
        // So v2 > v1.
        assert!(v2 > v1);

        // User scenario: "Outdated 1.19.0-pre.3 -> 1.18.4"
        // Current: 1.19.0-pre.3 (v2)
        // Latest (from repo): 1.18.4 (v1)
        // Check condition: if latest > current
        // if v1 > v2 { update } else { no update }
        assert!(v1 <= v2); // Should NOT suggest update

        // User scenario: "Outdated 1.19.2 -> v1.19.2"
        let v3 = parse_version("1.19.2").unwrap();
        let v4 = parse_version("v1.19.2").unwrap();
        assert_eq!(v3, v4);
        assert!(v3 <= v4); // Should NOT suggest update
    }
}
