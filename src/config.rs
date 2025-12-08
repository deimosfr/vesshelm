pub mod error;
use self::error::ConfigError;

use serde::{Deserialize, Serialize};
use serde_yaml_ng::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::{borrow::Cow, collections::HashSet};
use validator::{Validate, ValidationError};

#[derive(Debug, Deserialize, Serialize, Validate, Clone)]
#[validate(schema(function = "validate_config"))]
pub struct Config {
    #[validate(nested)]
    pub repositories: Vec<Repository>,
    #[validate(nested)]
    pub charts: Vec<Chart>,
    #[validate(nested)]
    pub destinations: Vec<Destination>,
    pub vesshelm: Option<VesshelmConfig>,
    pub ai: Option<AIConfig>,
    #[serde(alias = "variable_files")]
    pub variables_files: Option<Vec<String>>,
    #[serde(alias = "secret_files")]
    pub secrets_files: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Serialize, Validate, Clone)]
pub struct AIConfig {
    pub provider: AIProvider,
    pub api_key: Option<String>,
    pub model: Option<String>,
    pub endpoint: Option<String>,
    #[serde(default)]
    pub mode: AIMode,
    pub cli_binary: Option<String>,
    #[serde(default)]
    pub cli_args: Vec<String>,
    #[serde(default)]
    pub rtk: RTKConfig,
}

#[derive(Debug, Deserialize, Serialize, Default, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AIMode {
    #[default]
    Api,
    Cli,
}

#[derive(Debug, Deserialize, Serialize, Default, Clone)]
pub struct RTKConfig {
    #[serde(default = "default_true")]
    pub diff_compression: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AIProvider {
    Gemini,
    Claude,
    OpenAI,
    Ollama,
}

impl Config {
    pub fn load_from_path(path: &Path) -> Result<Self, ConfigError> {
        let content = fs::read_to_string(path)
            .map_err(|e| ConfigError::FileReadFailed(path.to_string_lossy().to_string(), e))?;
        let config: Config = serde_yaml_ng::from_str(&content).map_err(|e| {
            let msg = e.to_string();
            if msg.contains("missing field `url`") {
                return ConfigError::ParseFailedWithHint(
                    e,
                    "Check indentation of 'url' field. It must be indented to be part of the repository item.".to_string(),
                    path.to_string_lossy().to_string(),
                );
            }
            ConfigError::ParseFailed(e, path.to_string_lossy().to_string())
        })?;

        // validate config on load
        config.validate().map_err(ConfigError::ValidationFailed)?;
        Ok(config)
    }

    pub fn resolve_chart_destination(&self, chart: &Chart) -> Result<PathBuf, ConfigError> {
        match &chart.dest {
            Some(dest_val) => {
                // Try to find a named destination first
                if let Some(d) = self.destinations.iter().find(|d| &d.name == dest_val) {
                    Ok(PathBuf::from(&d.path))
                } else {
                    // If not found, treat as a direct path
                    Ok(PathBuf::from(dest_val))
                }
            }
            None => {
                let default_dest = self
                    .destinations
                    .iter()
                    .find(|d| d.name == "default")
                    .ok_or_else(|| ConfigError::DefaultDestNotFound(chart.name.clone()))?;
                Ok(PathBuf::from(&default_dest.path))
            }
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Validate, Clone)]
pub struct VesshelmConfig {
    pub helm_args: String,
    #[serde(default = "default_true")]
    pub diff_enabled: bool,
    #[serde(default)]
    pub diff_args: Option<String>,
    #[serde(default = "default_true")]
    pub deploy_debug_pause: bool,
    #[serde(default = "default_repo_update_cache_minutes")]
    pub repo_update_cache_minutes: u64,
}

fn default_true() -> bool {
    true
}

pub const DEFAULT_REPO_UPDATE_CACHE_MINUTES: u64 = 60;

fn default_repo_update_cache_minutes() -> u64 {
    DEFAULT_REPO_UPDATE_CACHE_MINUTES
}

fn validate_config(config: &Config) -> Result<(), ValidationError> {
    let repo_names: HashSet<&String> = config.repositories.iter().map(|r| &r.name).collect();
    let dest_names: HashSet<&String> = config.destinations.iter().map(|d| &d.name).collect();

    // check duplicates config
    if repo_names.len() != config.repositories.len() {
        return Err(ValidationError::new("duplicate_repository_names"));
    }
    if dest_names.len() != config.destinations.len() {
        return Err(ValidationError::new("duplicate_destination_names"));
    }

    // check variable files exist
    if let Some(files) = &config.variables_files {
        for file in files {
            if !std::path::Path::new(file).exists() {
                let mut err = ValidationError::new("variable_file_not_found");
                err.add_param(Cow::from("file"), file);
                return Err(err);
            }
        }
    }

    // check secrets files exist
    if let Some(files) = &config.secrets_files {
        for file in files {
            if !std::path::Path::new(file).exists() {
                let mut err = ValidationError::new("secrets_file_not_found");
                err.add_param(Cow::from("file"), file);
                return Err(err);
            }
        }
    }

    // check ai configuration
    if let Some(ai) = &config.ai
        && ai.mode == AIMode::Api
        && ai.provider != AIProvider::Ollama
        && ai.api_key.is_none()
    {
        let mut err = ValidationError::new("ai_api_key_missing");
        let provider_str = format!("{:?}", ai.provider);
        err.add_param(Cow::from("provider"), &provider_str);
        return Err(err);
    }

    // check duplicates charts
    let mut seen_charts = HashSet::new();

    for chart in &config.charts {
        let chart_key = (&chart.name, &chart.namespace);
        if !seen_charts.insert(chart_key) {
            let mut err = ValidationError::new("duplicate_chart_name_namespace");
            err.add_param(Cow::from("name"), &chart.name);
            err.add_param(Cow::from("namespace"), &chart.namespace);
            return Err(err);
        }

        match &chart.repo_name {
            Some(repo_name) if !repo_names.contains(repo_name) => {
                let mut err = ValidationError::new("chart_repo_not_found");
                err.add_param(Cow::from("chart_name"), &chart.name);
                err.add_param(Cow::from("repo_name"), repo_name);
                return Err(err);
            }
            _ => {}
        }

        if let Some(values_files) = &chart.values_files {
            for file in values_files {
                if !std::path::Path::new(file).exists() {
                    let mut err = ValidationError::new("values_file_not_found");
                    err.add_param(Cow::from("file"), file);
                    return Err(err);
                }
            }
        }

        if let Some(values) = &chart.values
            && !values.is_sequence()
            && !values.is_mapping()
        {
            let mut err = ValidationError::new("values_invalid_type");
            err.add_param(Cow::from("name"), &chart.name);
            return Err(err);
        }
    }
    Ok(())
}

fn validate_url_scheme(url: &str) -> Result<(), ValidationError> {
    // Simple scheme check
    if let Some(scheme_end) = url.find("://") {
        let scheme = &url[..scheme_end];
        match scheme {
            "http" | "https" | "file" | "git" | "ssh" | "oci" => Ok(()),
            _ => Err(ValidationError::new("invalid_url_scheme")),
        }
    } else {
        // If no scheme, valid if it's a file path?
        // But Repository URL implies scheme usually.
        // Let's assume invalid if no scheme like original validator might have done (except for file paths which might be just paths)
        // Note: validator::validate_url allows "example.com" (no scheme)?
        // No, it usually requires scheme.
        Err(ValidationError::new("missing_url_scheme"))
    }
}

#[derive(Debug, Deserialize, Serialize, Validate, Clone)]
pub struct Repository {
    #[validate(length(min = 1))]
    pub name: String,
    #[validate(custom(function = "validate_url_scheme"))]
    pub url: String,
    #[serde(default)]
    pub r#type: RepoType,
}

#[derive(Debug, Deserialize, Serialize, Default, PartialEq, Clone)]
#[serde(rename_all = "lowercase")]
pub enum RepoType {
    #[default]
    Helm,
    Git,
    Oci,
}

#[derive(Debug, Deserialize, Serialize, Validate, Clone)]
pub struct Chart {
    #[validate(length(min = 1))]
    pub name: String,
    pub repo_name: Option<String>,
    pub release_name: Option<String>,
    pub version: Option<String>,
    pub namespace: String,
    #[serde(alias = "destination_override")]
    pub dest: Option<String>,
    pub chart_path: Option<String>,
    #[serde(default)]
    pub no_sync: bool,
    #[serde(default)]
    pub no_deploy: bool,
    pub comment: Option<String>,
    pub values_files: Option<Vec<String>>,
    pub helm_args_append: Option<String>,
    pub helm_args_override: Option<String>,
    pub values: Option<Value>,
    pub depends: Option<Vec<String>>,
    #[serde(default)]
    pub no_interpolation: bool,
}

impl Chart {
    pub fn release_name(&self) -> String {
        self.release_name
            .as_deref()
            .unwrap_or(&self.name)
            .to_string()
    }

    pub fn selection_label(&self) -> String {
        format!("{} ({})", self.name, self.namespace)
    }
}

#[cfg(test)]
impl Default for Chart {
    fn default() -> Self {
        Self {
            name: "test-chart".to_string(),
            repo_name: Some("test-repo".to_string()),
            release_name: None,
            version: Some("1.0.0".to_string()),
            namespace: "default".to_string(),
            dest: None,
            chart_path: None,
            no_sync: false,
            no_deploy: false,
            comment: None,
            values_files: None,
            helm_args_append: None,
            helm_args_override: None,
            values: None,
            depends: None,
            no_interpolation: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ai_config() {
        let yaml = r#"
repositories: []
charts: []
destinations: []
ai:
  provider: gemini
  api_key: "test-key"
  rtk:
    diff_compression: true
"#;
        let config: Config = serde_yaml_ng::from_str(yaml).unwrap();
        assert!(config.ai.is_some());
        let ai = config.ai.unwrap();
        assert_eq!(ai.provider, AIProvider::Gemini);
        assert_eq!(ai.api_key.unwrap(), "test-key");
        assert!(ai.rtk.diff_compression);
    }

    #[test]
    fn test_ai_config_validation() {
        let yaml = r#"
repositories: []
charts: []
destinations: []
ai:
  provider: gemini
"#;
        let config: Config = serde_yaml_ng::from_str(yaml).unwrap();
        let result = config.validate();
        assert!(result.is_err());
    }
}

#[derive(Debug, Deserialize, Serialize, Validate, Clone)]
pub struct Destination {
    #[validate(length(min = 1))]
    pub name: String,
    #[validate(length(min = 1))]
    pub path: String,
}
