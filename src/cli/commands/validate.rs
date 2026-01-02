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
    let config: Config = serde_yaml_ng::from_str(&content)?;

    if let Err(e) = config.validate() {
        for (field, error_kind) in e.errors() {
            match error_kind {
                validator::ValidationErrorsKind::Field(errors) => {
                    for err in errors {
                        if *field == "__all__" {
                            print_validation_error(err);
                        } else {
                            eprintln!("  - Field '{}':", style(field).yellow());
                            print_validation_error(err);
                        }
                    }
                }
                _ => {
                    eprintln!("  - {}: {:?}", field, error_kind);
                }
            }
        }
        anyhow::bail!("Validation failed");
    }

    fn print_validation_error(err: &validator::ValidationError) {
        eprint!("    {} {}", style("➜").red(), style(&err.code).bold());
        if !err.params.is_empty() {
            eprint!(" (");
            let mut first = true;
            for (key, value) in &err.params {
                if !first {
                    eprint!(", ");
                }
                eprint!("{}: {}", key, value);
                first = false;
            }
            eprint!(")");
        }
        if let Some(msg) = &err.message {
            eprint!(": {}", msg);
        }
        eprintln!();
    }

    println!("{}", "Configuration is valid".green());
    Ok(())
}
