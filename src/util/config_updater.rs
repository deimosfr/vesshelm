use crate::config::{RepoType, Repository};
use anyhow::{Context, Result, anyhow};
use regex::Regex;
use std::path::Path;

pub struct ConfigUpdater;

impl ConfigUpdater {
    pub fn update(config_path: &Path, repo: Option<Repository>, chart: ChartConfig) -> Result<()> {
        let mut file_content =
            std::fs::read_to_string(config_path).context("Failed to read config file")?;

        if let Some(r) = repo {
            Self::add_repository(&mut file_content, &r);
        }

        Self::add_chart(&mut file_content, &chart);

        std::fs::write(config_path, file_content).context("Failed to write config file")?;
        Ok(())
    }

    fn add_repository(content: &mut String, r: &Repository) {
        let mut repo_block = format!("\n  - name: {}\n    url: {}", r.name, r.url);
        if r.r#type != RepoType::Helm {
            repo_block.push_str(&format!("\n    type: {:?}", r.r#type).to_lowercase());
        }

        if let Some(idx) = content.find("repositories:") {
            let insert_at = idx + "repositories:".len();
            content.insert_str(insert_at, &repo_block);
        } else {
            content.push_str("\nrepositories:");
            content.push_str(&repo_block);
        }
    }

    fn add_chart(content: &mut String, chart: &ChartConfig) {
        let mut chart_block = format!(
            "\n  - name: {}\n    repo_name: {}\n    namespace: {}",
            chart.name, chart.repo_name, chart.namespace
        );

        if let Some(v) = &chart.version {
            chart_block.push_str(&format!("\n    version: {}", v));
        }

        if let Some(path) = &chart.chart_path {
            chart_block.push_str(&format!("\n    chart_path: {}", path));
        }

        if let Some(comment) = &chart.comment {
            chart_block.push_str(&format!("\n    comment: {}", comment));
        }

        if let Some(idx) = content.find("charts:") {
            let insert_at = idx + "charts:".len();
            content.insert_str(insert_at, &chart_block);
        } else {
            content.push_str("\ncharts:");
            content.push_str(&chart_block);
        }
    }

    pub fn update_chart_version(
        config_path: &Path,
        chart_name: &str,
        new_version: &str,
    ) -> Result<()> {
        let mut file_content =
            std::fs::read_to_string(config_path).context("Failed to read config file")?;

        Self::replace_chart_version_in_text(&mut file_content, chart_name, new_version)?;

        std::fs::write(config_path, file_content).context("Failed to write updated config")?;
        Ok(())
    }

    pub fn replace_chart_version_in_text(
        content: &mut String,
        chart_name: &str,
        new_version: &str,
    ) -> Result<()> {
        // 1. Find the line that starts the chart block: `name: <chart_name>`
        let name_regex = Regex::new(&format!(
            r"(?m)^(\s*-\s*|\s*)name:\s*{}\s*$",
            regex::escape(chart_name)
        ))?;

        let mat = name_regex
            .find(content)
            .ok_or_else(|| anyhow!("Chart {} not found", chart_name))?;
        let start_idx = mat.end(); // We start search after the name line

        // 2. Scan lines after the name to find `version:`
        let version_regex = Regex::new(r"(?m)^(\s*)version:\s*([^\s]+)")?;

        let suffix = &content[start_idx..];

        // Find first version match
        if let Some(v_cap) = version_regex.captures(suffix) {
            // Check if there is a "danger marker" before this match.
            let match_start_in_suffix = v_cap.get(0).unwrap().start();
            let pre_match_text = &suffix[..match_start_in_suffix];

            // Check if pre_match_text contains a new list item marker `- name:` or `- ` at base level.
            let danger_regex = Regex::new(r"(?m)^\s*(-\s*)?name:")?;
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
}

pub struct ChartConfig {
    pub name: String,
    pub repo_name: String,
    pub namespace: String,
    pub version: Option<String>,
    pub chart_path: Option<String>,
    pub comment: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{RepoType, Repository};

    #[test]
    fn test_add_repository_empty() {
        let mut content = String::new();
        let repo = Repository {
            name: "test-repo".to_string(),
            url: "https://example.com".to_string(),
            r#type: RepoType::Helm,
        };
        ConfigUpdater::add_repository(&mut content, &repo);
        assert!(content.contains("repositories:"));
        assert!(content.contains("- name: test-repo"));
        assert!(content.contains("url: https://example.com"));
    }

    #[test]
    fn test_add_repository_append() {
        let mut content = "repositories:\n  - name: existing\n    url: http://ex.com".to_string();
        let repo = Repository {
            name: "new-repo".to_string(),
            url: "git://github.com/foo/bar.git".to_string(),
            r#type: RepoType::Git,
        };
        ConfigUpdater::add_repository(&mut content, &repo);
        assert!(content.contains("type: git"));
        // Check order loosely or just existence
        let lines: Vec<&str> = content.lines().collect();
        assert!(lines.len() > 3);
    }

    #[test]
    fn test_add_chart() {
        let mut content = "charts:\n".to_string();
        let chart = ChartConfig {
            name: "my-chart".to_string(),
            repo_name: "my-repo".to_string(),
            namespace: "default".to_string(),
            version: Some("1.2.3".to_string()),
            chart_path: Some("charts/foo".to_string()),
            comment: None,
        };
        ConfigUpdater::add_chart(&mut content, &chart);
        assert!(content.contains("- name: my-chart"));
        assert!(content.contains("repo_name: my-repo"));
        assert!(content.contains("version: 1.2.3"));
        assert!(content.contains("chart_path: charts/foo"));
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

        ConfigUpdater::replace_chart_version_in_text(&mut yaml_content, chart_name, new_version)
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

        ConfigUpdater::replace_chart_version_in_text(&mut yaml_content_2, "my-chart", "v1.2.4")
            .expect("replacement failed");
        assert!(yaml_content_2.contains("version: v1.2.4"));
        assert!(!yaml_content_2.contains("version: v1.2.3"));
    }
}
