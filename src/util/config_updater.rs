use crate::config::error::ConfigError;
use crate::config::{RepoType, Repository};
use regex::Regex;
use std::path::Path;

pub struct ConfigUpdater;

impl ConfigUpdater {
    pub fn update(
        config_path: &Path,
        repo: Option<Repository>,
        chart: ChartConfig,
    ) -> Result<(), ConfigError> {
        let mut file_content = std::fs::read_to_string(config_path).map_err(|e| {
            ConfigError::FileReadFailed(config_path.to_string_lossy().to_string(), e)
        })?;

        if let Some(r) = repo {
            Self::add_repository(&mut file_content, &r);
        }

        Self::add_chart(&mut file_content, &chart);

        std::fs::write(config_path, file_content).map_err(|e| {
            ConfigError::FileWriteFailed(config_path.to_string_lossy().to_string(), e)
        })?;
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
    ) -> Result<(), ConfigError> {
        let mut file_content = std::fs::read_to_string(config_path).map_err(|e| {
            ConfigError::FileReadFailed(config_path.to_string_lossy().to_string(), e)
        })?;

        Self::replace_chart_version_in_text(&mut file_content, chart_name, new_version)?;

        std::fs::write(config_path, file_content).map_err(|e| {
            ConfigError::FileWriteFailed(config_path.to_string_lossy().to_string(), e)
        })?;
        Ok(())
    }

    /// Updates the version for one chart identity without affecting another
    /// chart that happens to share the same name in a different namespace.
    pub fn update_chart_version_for_namespace(
        config_path: &Path,
        chart_name: &str,
        namespace: &str,
        new_version: &str,
    ) -> Result<(), ConfigError> {
        let mut file_content = std::fs::read_to_string(config_path).map_err(|e| {
            ConfigError::FileReadFailed(config_path.to_string_lossy().to_string(), e)
        })?;

        Self::replace_chart_version_for_namespace_in_text(
            &mut file_content,
            chart_name,
            namespace,
            new_version,
        )?;

        std::fs::write(config_path, file_content).map_err(|e| {
            ConfigError::FileWriteFailed(config_path.to_string_lossy().to_string(), e)
        })?;
        Ok(())
    }

    pub fn replace_chart_version_in_text(
        content: &mut String,
        chart_name: &str,
        new_version: &str,
    ) -> Result<(), ConfigError> {
        // 1. Find the line that starts the chart block: `name: <chart_name>`
        let name_regex = Regex::new(&format!(
            r"(?m)^(\s*-\s*|\s*)name:\s*{}\s*$",
            regex::escape(chart_name)
        ))?;

        let mat = name_regex
            .find(content)
            .ok_or_else(|| ConfigError::ChartNotFound(chart_name.to_string()))?;
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
                return Err(ConfigError::VersionBoundaryCrossed(chart_name.to_string()));
            }

            // Safe to replace
            let version_val_match = v_cap
                .get(2) // Group 2 is value
                .ok_or(ConfigError::RegexCaptureFailed(format!(
                    "Failed to capture version for chart {}",
                    chart_name
                )))?;
            let range = version_val_match.range();

            // Calculate absolute range in original content
            let abs_start = start_idx + range.start;
            let abs_end = start_idx + range.end;

            content.replace_range(abs_start..abs_end, new_version);
            return Ok(());
        }

        Err(ConfigError::VersionFieldNotFound(chart_name.to_string()))
    }

    pub fn replace_chart_version_for_namespace_in_text(
        content: &mut String,
        chart_name: &str,
        namespace: &str,
        new_version: &str,
    ) -> Result<(), ConfigError> {
        let name_regex = Regex::new(&format!(
            r"(?m)^(?P<indent>\s*)-\s*name:\s*{}\s*$",
            regex::escape(chart_name)
        ))?;
        let namespace_regex = Regex::new(&format!(
            r"(?m)^\s*namespace:\s*{}\s*$",
            regex::escape(namespace)
        ))?;
        let version_regex = Regex::new(r"(?m)^(\s*)version:\s*([^\s]+)")?;

        for captures in name_regex.captures_iter(content) {
            let name_match = captures.get(0).ok_or(ConfigError::RegexCaptureFailed(
                "Failed to capture chart name block".to_string(),
            ))?;
            let indent = captures
                .name("indent")
                .ok_or(ConfigError::RegexCaptureFailed(
                    "Failed to capture chart indentation".to_string(),
                ))?
                .as_str();
            let block_start = name_match.end();
            let next_item_regex = Regex::new(&format!(r"(?m)^{}-\s*name:", regex::escape(indent)))?;
            let block_end = next_item_regex
                .find(&content[block_start..])
                .map_or(content.len(), |next| block_start + next.start());
            let block = &content[block_start..block_end];

            if !namespace_regex.is_match(block) {
                continue;
            }

            let version_match = version_regex
                .captures(block)
                .and_then(|captures| captures.get(2));
            let Some(version_match) = version_match else {
                return Err(ConfigError::VersionFieldNotFound(format!(
                    "{chart_name} ({namespace})"
                )));
            };
            let range = version_match.range();
            content.replace_range(
                (block_start + range.start)..(block_start + range.end),
                new_version,
            );
            return Ok(());
        }

        Err(ConfigError::ChartNotFound(format!(
            "{chart_name} ({namespace})"
        )))
    }
    pub fn remove_chart(
        config_path: &Path,
        chart_name: &str,
        namespace: &str,
    ) -> Result<(), ConfigError> {
        let file_content = std::fs::read_to_string(config_path).map_err(|e| {
            ConfigError::FileReadFailed(config_path.to_string_lossy().to_string(), e)
        })?;

        let new_content = Self::remove_list_item_by_fields(
            &file_content,
            "charts",
            &[("name", chart_name), ("namespace", namespace)],
        )?;

        if file_content != new_content {
            std::fs::write(config_path, new_content).map_err(|e| {
                ConfigError::FileWriteFailed(config_path.to_string_lossy().to_string(), e)
            })?;
        }
        Ok(())
    }

    pub fn remove_repository(config_path: &Path, repo_name: &str) -> Result<(), ConfigError> {
        let file_content = std::fs::read_to_string(config_path).map_err(|e| {
            ConfigError::FileReadFailed(config_path.to_string_lossy().to_string(), e)
        })?;

        let new_content = Self::remove_list_item_by_fields(
            &file_content,
            "repositories",
            &[("name", repo_name)],
        )?;

        if file_content != new_content {
            std::fs::write(config_path, new_content).map_err(|e| {
                ConfigError::FileWriteFailed(config_path.to_string_lossy().to_string(), e)
            })?;
        }
        Ok(())
    }

    /// Generic helper to remove a list item that matches specific field values.
    fn remove_list_item_by_fields(
        content: &str,
        section_key: &str,
        fields: &[(&str, &str)], // (key, value) pairs that must ALL match
    ) -> Result<String, ConfigError> {
        let lines: Vec<&str> = content.lines().collect();

        // 1. Find section start
        let section_idx = lines
            .iter()
            .position(|l| l.trim_start().starts_with(&format!("{}:", section_key)));
        if section_idx.is_none() {
            return Ok(content.to_string());
        }
        let section_idx = section_idx.unwrap();

        // 2. Iterate items
        let mut i = section_idx + 1;
        let mut lines_to_remove = Vec::new();

        while i < lines.len() {
            let line = lines[i];
            // Check if end of section (dedent or new key at same level)
            // Assuming section key is top level or indented.
            // We assume standard 2-space indentation for list items usually, so if we see something with same indent as section_key, we break.
            // But let's check indent.
            let section_indent = lines[section_idx]
                .find(|c: char| !c.is_whitespace())
                .unwrap_or(0);
            let current_indent = line.find(|c: char| !c.is_whitespace()).unwrap_or(0);

            // Skip comments and empty lines in the main loop checks
            // We don't want a comment at the same level as the section key to break the section.
            let trimmed_line = line.trim();
            if trimmed_line.is_empty() || trimmed_line.starts_with('#') {
                i += 1;
                continue;
            }

            if current_indent < section_indent
                || (current_indent == section_indent && !trimmed_line.starts_with("-"))
            {
                // End of section (non-comment line at lower indent, or same indent but not a list item)
                break;
            }

            // Check if list item start
            if line.trim_start().starts_with("-") {
                let item_start = i;
                let mut item_end = i + 1;
                let item_indent = current_indent; // The indent of the dash

                // Find end of this item (next item or end of section)
                while item_end < lines.len() {
                    let next_line = lines[item_end];

                    let trimmed_next = next_line.trim_start();

                    if trimmed_next.is_empty() {
                        // Empty line. Look ahead to see if the next non-empty line acts as a separator (e.g., a header comment).
                        let mut lookahead = item_end + 1;
                        let mut found_header = false;
                        while lookahead < lines.len() {
                            let l = lines[lookahead].trim_start();
                            if l.is_empty() {
                                lookahead += 1;
                                continue;
                            }
                            // If we see a comment block after empty lines, we assume it's a section header or separate block.
                            // STOPS consumption.
                            if l.starts_with('#') {
                                found_header = true;
                            }
                            // We stop looking after finding the first non-empty line
                            break;
                        }

                        if found_header {
                            // The empty line is followed by a comment (header).
                            // We should stop here.
                            break;
                        }

                        // Standard empty line consumption (within item or between items without header)
                        item_end += 1;
                        continue;
                    }

                    if trimmed_next.starts_with('#') {
                        // Comment immediately following (no empty line).
                        // Likely belongs to the item.
                        item_end += 1;
                        continue;
                    }

                    let next_indent = next_line.find(|c: char| !c.is_whitespace()).unwrap_or(0);

                    if next_indent <= section_indent {
                        break; // End of section (dedent or same level key)
                    }
                    if next_indent == item_indent && trimmed_next.starts_with('-') {
                        break; // Next item
                    }

                    item_end += 1;
                }

                // Check matches in this block [item_start, item_end)
                // We construct a mini-string to parse? No, just regex check line by line.
                // Careful: indentation.
                let block_lines = &lines[item_start..item_end];
                let mut all_matched = true;

                for (key, val) in fields {
                    // Regex find `key:\s*val`
                    // Just simple string contains check might fail if commented out.
                    // Let's use Regex.
                    // We assume values don't span multiple lines for now (names usually don't).
                    // Regex to match `key: value` with optional quotes and comments
                    let re = Regex::new(&format!(
                        r"(?m)^\s*(- )?{}:\s*['\x22]?{}['\x22]?\s*(#.*)?$",
                        regex::escape(key),
                        regex::escape(val)
                    ))?;

                    let found = block_lines.iter().any(|l| re.is_match(l));
                    if !found {
                        all_matched = false;
                        break;
                    }
                }

                if all_matched {
                    // Mark these lines for removal
                    for k in item_start..item_end {
                        lines_to_remove.push(k);
                    }
                    // Break loop after finding and removing one matching item?
                    // Original code removed and returned.
                    // Let's stick to original behavior (remove first match and return) or should we remove all?
                    // Original code did: return Ok(...)
                    // So it removed one and returned.

                    let mut new_lines = lines.clone();
                    new_lines.drain(item_start..item_end);
                    return Ok(
                        new_lines.join("\n") + if content.ends_with("\n") { "\n" } else { "" }
                    );
                }

                // Move i to next item
                i = item_end;
                continue;
            }

            i += 1;
        }

        Ok(content.to_string())
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

    #[test]
    fn test_remove_list_item() {
        let content = r#"# start
repositories:
  - name: repo1
    url: url1
  - name: repo2
    url: url2
charts:
  - name: chart1
    namespace: ns1
    # comment
  - name: chart2
    namespace: ns2
    # comment 2
"#;

        // Test removing chart1
        let res =
            ConfigUpdater::remove_list_item_by_fields(content, "charts", &[("name", "chart1")])
                .unwrap();
        assert!(!res.contains("name: chart1"));
        assert!(res.contains("name: chart2"));
        assert!(res.contains("# comment 2"));
        // Check formatting preservation
        assert!(res.starts_with("# start\nrepositories:"));

        // Test removing repo2
        let res2 = ConfigUpdater::remove_list_item_by_fields(
            content,
            "repositories",
            &[("name", "repo2")],
        )
        .unwrap();
        assert!(!res2.contains("name: repo2"));
        assert!(res2.contains("name: repo1"));
        assert!(res2.contains("url: url1"));

        // Test removing chart with multiple fields
        let res3 = ConfigUpdater::remove_list_item_by_fields(
            content,
            "charts",
            &[("name", "chart2"), ("namespace", "ns2")],
        )
        .unwrap();
        assert!(!res3.contains("name: chart2"));
        assert!(res3.contains("name: chart1"));
    }

    #[test]
    fn test_remove_list_item_with_quotes() {
        let content = r#"charts:
  - name: "chart-quoted"
    namespace: 'ns-quoted'
  - name: chart-normal
"#;
        let res = ConfigUpdater::remove_list_item_by_fields(
            content,
            "charts",
            &[("name", "chart-quoted"), ("namespace", "ns-quoted")],
        )
        .unwrap();

        assert!(!res.contains("chart-quoted"));
        assert!(res.contains("chart-normal"));
    }
    #[test]
    fn test_remove_list_item_with_comments_and_headers() {
        let content = r#"charts:

########################
# Network
########################

  - name: chart1
    namespace: ns1

########################
# Misc
########################

  - name: chart2
    namespace: ns2
"#;
        // Try to remove chart2, which is after a header
        let res =
            ConfigUpdater::remove_list_item_by_fields(content, "charts", &[("name", "chart2")])
                .unwrap();

        assert!(!res.contains("name: chart2"));
        assert!(res.contains("name: chart1"));

        // Try to remove chart1, which is after a header relative to "charts:"
        let res2 =
            ConfigUpdater::remove_list_item_by_fields(content, "charts", &[("name", "chart1")])
                .unwrap();
        assert!(!res2.contains("name: chart1"));
        assert!(res2.contains("name: chart2"));
    }

    #[test]
    fn test_remove_list_item_preserves_headers_with_newline() {
        let content = r#"charts:
  - name: chart1
    version: 1.0.0

########################
# Misc
########################

  - name: chart2
"#;
        let res =
            ConfigUpdater::remove_list_item_by_fields(content, "charts", &[("name", "chart1")])
                .unwrap();

        assert!(!res.contains("name: chart1"));
        assert!(res.contains("########################"));
        assert!(res.contains("# Misc"));
        assert!(res.contains("name: chart2"));
    }

    #[test]
    fn test_remove_chart_io() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        let mut temp = NamedTempFile::new().unwrap();
        let content = r#"charts:
  - name: to-remove
    namespace: default
  - name: keep
    namespace: default
"#;
        temp.write_all(content.as_bytes()).unwrap();

        ConfigUpdater::remove_chart(temp.path(), "to-remove", "default").unwrap();

        let new_content = std::fs::read_to_string(temp.path()).unwrap();
        assert!(!new_content.contains("name: to-remove"));
        assert!(new_content.contains("name: keep"));
    }

    #[test]
    fn test_remove_repository_io() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        let mut temp = NamedTempFile::new().unwrap();
        let content = r#"repositories:
  - name: repo-remove
    url: http://foo.bar
  - name: repo-keep
    url: http://keep.me
"#;
        temp.write_all(content.as_bytes()).unwrap();

        ConfigUpdater::remove_repository(temp.path(), "repo-remove").unwrap();

        let new_content = std::fs::read_to_string(temp.path()).unwrap();
        assert!(!new_content.contains("name: repo-remove"));
        assert!(new_content.contains("name: repo-keep"));
    }

    #[test]
    fn update_chart_version_for_namespace_only_changes_the_target_chart() {
        let mut content = r#"charts:
  - name: nginx
    namespace: staging
    version: 1.25.0
  - name: nginx
    namespace: production
    version: 1.24.0
"#
        .to_string();

        ConfigUpdater::replace_chart_version_for_namespace_in_text(
            &mut content,
            "nginx",
            "production",
            "1.26.0",
        )
        .expect("target chart version should update");

        assert!(content.contains("namespace: staging\n    version: 1.25.0"));
        assert!(content.contains("namespace: production\n    version: 1.26.0"));
    }
}
