use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_yaml_ng::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::{borrow::Cow, collections::HashSet};
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
    pub vesshelm: Option<VesshelmConfig>,
    pub variable_files: Option<Vec<String>>,
}

impl Config {
    pub fn load_from_path(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read configuration file: {:?}", path))?;
        let config: Config = serde_yaml_ng::from_str(&content)?;

        // validate config on load
        config
            .validate()
            .context("Configuration validation failed")?;
        Ok(config)
    }

    pub fn resolve_chart_destination(&self, chart: &Chart) -> Result<PathBuf> {
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
                    .ok_or_else(|| {
                        anyhow::anyhow!("Default destination 'default' not found in configuration")
                    })?;
                Ok(PathBuf::from(&default_dest.path))
            }
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct VesshelmConfig {
    pub helm_args: String,
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

    // check duplicates config
    if repo_names.len() != config.repositories.len() {
        return Err(ValidationError::new("duplicate_repository_names"));
    }
    if dest_names.len() != config.destinations.len() {
        return Err(ValidationError::new("duplicate_destination_names"));
    }

    // check variable files exist
    if let Some(files) = &config.variable_files {
        for file in files {
            if !std::path::Path::new(file).exists() {
                let mut err = ValidationError::new("variable_file_not_found");
                err.add_param(Cow::from("file"), file);
                return Err(err);
            }
        }
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
                return Err(ValidationError::new("chart_repo_not_found"));
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

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct Repository {
    #[validate(length(min = 1))]
    pub name: String,
    #[validate(custom(function = "validate_url_scheme"))]
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
