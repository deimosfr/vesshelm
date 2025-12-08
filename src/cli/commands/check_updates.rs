use anyhow::{Context, Result, anyhow};
use colored::*;
use console::style;
use semver::Version;
use vesshelm::config::{Config, RepoType};

use std::collections::HashMap;
use std::process::Command;
use tokio::fs;

use super::CheckUpdatesArgs;

pub async fn run(args: CheckUpdatesArgs, config_path: &std::path::Path) -> Result<()> {
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
        // Filter by --only arg
        if let Some(only) = &args.only
            && !only.contains(&chart.name)
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

    match (args.apply, updates_found) {
        (true, true) => {
            println!("\n{} Applying updates...", "📝".bold());

            // Apply updates non-destructively
            let updates_map: HashMap<String, String> = charts_to_update.into_iter().collect();
            apply_updates_non_destructive(config_path, &updates_map).await?;

            println!("{} vesshelm.yaml updated.", "✅".green());
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

async fn apply_updates_non_destructive(
    config_path: &std::path::Path,
    updates: &HashMap<String, String>,
) -> Result<()> {
    // Read the entire file
    let content = fs::read_to_string(config_path)
        .await
        .context("Failed to read config file")?;

    let mut new_content = content.clone();

    for (chart_name, new_version) in updates {
        if let Err(e) = find_and_replace_version(&mut new_content, chart_name, new_version) {
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

    // Write back only if changes were made (though we iterate updates so strictly yes)
    if new_content != content {
        fs::write(config_path, new_content)
            .await
            .context("Failed to write updated config")?;
    }

    Ok(())
}

fn find_and_replace_version(
    content: &mut String,
    chart_name: &str,
    new_version: &str,
) -> Result<()> {
    // 1. Find the line that starts the chart block: `name: <chart_name>`
    let name_regex = regex::Regex::new(&format!(
        r"(?m)^(\s*-\s*|\s*)name:\s*{}\s*$",
        regex::escape(chart_name)
    ))?;

    let mat = name_regex
        .find(content)
        .ok_or_else(|| anyhow!("Chart {} not found", chart_name))?;
    let start_idx = mat.end(); // We start search after the name line

    // 2. Scan lines after the name to find `version:`
    let version_regex = regex::Regex::new(r"(?m)^(\s*)version:\s*([^\s]+)")?;

    let suffix = &content[start_idx..];

    // Find first version match
    if let Some(v_cap) = version_regex.captures(suffix) {
        // v_cap(0) is the whole line/match "  version: 1.2.3"
        // Check if there is a "danger marker" before this match.
        let match_start_in_suffix = v_cap
            .get(0)
            .ok_or_else(|| anyhow!("Failed to get capture group 0"))?
            .start();
        let pre_match_text = &suffix[..match_start_in_suffix];

        // Check if pre_match_text contains a new list item marker `- name:` or `- ` at base level.
        let danger_regex = regex::Regex::new(r"(?m)^\s*(-\s*)?name:")?;
        if danger_regex.is_match(pre_match_text) {
            return Err(anyhow!(
                "Found version field but it belongs to another chart (crossed boundary)"
            ));
        }

        // Safe to replace
        let version_val_match = v_cap
            .get(2) // Group 2 is value
            .ok_or_else(|| anyhow!("Failed to get version value capture group"))?;
        let range = version_val_match.range();

        // Calculate absolute range in original content
        let abs_start = start_idx + range.start;
        let abs_end = start_idx + range.end;

        content.replace_range(abs_start..abs_end, new_version);
        return Ok(());
    }

    Err(anyhow!(
        "Could not find version field for chart {}",
        chart_name
    ))
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

    #[test]
    fn test_verify_regex_replacement() {
        let mut yaml_content = r#"
charts:
  - name: chart1
    repo_name: repo1
    version: 1.0.0 # comment
    other: field
  - name: chart2
    version: 2.0.0
    # comment line
    some: value
"#
        .to_string();

        let chart_name = "chart1";
        let new_version = "1.0.1";

        find_and_replace_version(&mut yaml_content, chart_name, new_version)
            .expect("replacement failed");

        // Verify changes
        assert!(yaml_content.contains("version: 1.0.1 # comment"));
        // Verify other chart unchanged
        assert!(yaml_content.contains("version: 2.0.0"));
        // Verify structure preserved
        assert!(yaml_content.contains("other: field"));

        // Test tricky formatting
        let mut yaml_content_2 = r#"
charts:
  - name: my-chart
    version: v1.2.3
  - name: other
    version: 1.0.0
"#
        .to_string();

        find_and_replace_version(&mut yaml_content_2, "my-chart", "v1.2.4")
            .expect("replacement failed");
        assert!(yaml_content_2.contains("version: v1.2.4"));
        assert!(!yaml_content_2.contains("version: v1.2.3"));
    }
}
