use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_yaml_ng::Value;
use std::collections::HashSet;
use std::fs;
use std::path::Path;
use validator::{Validate, ValidationError};

#[derive(Debug, Deserialize, Serialize, Validate)]
#[validate(schema(function = "validate_config"))]
pub struct Config {
    #[validate(nested)]
    pub repositories: Vec<Repository>,
    #[validate(nested)]
    pub charts: Vec<Chart>,
    #[validate(nested)]
    pub destinations: Vec<Destination>,
    pub helm: Option<HelmConfig>,
}

impl Config {
    pub fn load_from_path(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read configuration file: {:?}", path))?;
        let config: Config = serde_yaml_ng::from_str(&content)
            .with_context(|| "Failed to parse configuration file")?;
        // Validate immediately upon load?
        // Existing commands do: load -> parse -> validate.
        // sync.rs does explicit validate() call. deploy.rs DOES NOT do explicit validate() call in the snippet I saw?
        // Let's check deploy.rs again.
        // It does: `let config: Config = serde_yaml::from_str...`. It DOES NOT call `config.validate()`.
        // sync.rs DOES `config.validate()`.
        // validate.rs DOES `config.validate()`.
        // It seems safer to validate on load, but if deploy.rs didn't, maybe it was an oversight?
        // The `validate` derive macro is there.
        // The Proposal requirements say: "All commands that read ...".
        // Let's include validation in load_from_path to enforce it everywhere, unless there's dynamic modification?
        // No dynamic mod properly supported yet.
        // So yes, validate.

        // However, `validate` returns `Result<(), ValidationError>`.
        // Need to make sure `ValidationError` implements `std::error::Error` so `anyhow` can wrap it. It usually does.
        config
            .validate()
            .context("Configuration validation failed")?;
        Ok(config)
    }
}

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct HelmConfig {
    pub args: String,
    #[serde(default = "default_true")]
    pub diff_enabled: bool,
    #[serde(default)]
    pub diff_args: Option<String>,
}

fn default_true() -> bool {
    true
}

fn validate_config(config: &Config) -> Result<(), ValidationError> {
    let repo_names: HashSet<&String> = config.repositories.iter().map(|r| &r.name).collect();
    let dest_names: HashSet<&String> = config.destinations.iter().map(|d| &d.name).collect();

    if repo_names.len() != config.repositories.len() {
        return Err(ValidationError::new("duplicate_repository_names"));
    }
    if dest_names.len() != config.destinations.len() {
        return Err(ValidationError::new("duplicate_destination_names"));
    }

    let mut seen_charts = HashSet::new();

    for chart in &config.charts {
        let chart_key = (&chart.name, &chart.namespace);
        if !seen_charts.insert(chart_key) {
            let mut err = ValidationError::new("duplicate_chart_name_namespace");
            err.add_param(std::borrow::Cow::from("name"), &chart.name);
            err.add_param(std::borrow::Cow::from("namespace"), &chart.namespace);
            return Err(err);
        }

        match &chart.repo_name {
            Some(repo_name) if !repo_names.contains(repo_name) => {
                return Err(ValidationError::new("chart_repo_not_found"));
            }
            _ => {}
        }
        match &chart.dest {
            Some(dest) if !dest_names.contains(dest) => {
                return Err(ValidationError::new("chart_dest_not_found"));
            }
            _ => {}
        }

        if let Some(values_files) = &chart.values_files {
            for file in values_files {
                if !std::path::Path::new(file).exists() {
                    let mut err = ValidationError::new("values_file_not_found");
                    err.add_param(std::borrow::Cow::from("file"), file);
                    return Err(err);
                }
            }
        }
    }
    Ok(())
}

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct Repository {
    #[validate(length(min = 1))]
    pub name: String,
    #[validate(url)]
    pub url: String,
    #[serde(default)]
    pub r#type: RepoType,
}

#[derive(Debug, Deserialize, Serialize, Default, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum RepoType {
    #[default]
    Helm,
    Git,
    Oci,
}

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct Chart {
    #[validate(length(min = 1))]
    pub name: String,
    pub repo_name: Option<String>,
    pub version: Option<String>,
    pub namespace: String,
    #[serde(rename = "destination_override")]
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
    pub values: Option<Vec<Value>>,
    pub depends: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct Destination {
    #[validate(length(min = 1))]
    pub name: String,
    #[validate(length(min = 1))]
    pub path: String,
}
