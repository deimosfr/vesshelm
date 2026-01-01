use crate::config::Config;
use anyhow::{Context, Result};
use colored::*;
use console::style;
use std::fs;
use std::path::Path;
use validator::Validate; // Needed for trait method

pub async fn run(config_path: &Path) -> Result<()> {
    println!("{} Validating configuration", style("==>").bold().green());

    let content = fs::read_to_string(config_path)
        .with_context(|| format!("Failed to read configuration file: {:?}", config_path))?;
    let config: Config =
        serde_yaml_ng::from_str(&content).context("Failed to parse configuration file")?;

    config
        .validate()
        .context("Configuration validation failed")?;

    println!("{}", "Configuration is valid".green());
    Ok(())
}
