use console::style;
use thiserror::Error;
use validator::{ValidationErrors, ValidationErrorsKind};

#[derive(Error, Debug)]
pub enum VariablesError {
    #[error("File not found: {0}")]
    FileNotFound(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Failed to parse YAML from '{0}': {1}")]
    ParseYamlFailed(String, #[source] serde_yaml_ng::Error),
    #[error("Template rendering failed for '{0}': {1}")]
    TemplateRenderFailed(String, #[source] minijinja::Error),
    #[error("Decryption failed: {0}")]
    DecryptionFailed(String),
    #[error("Encryption error: {0}")]
    Encryption(#[from] EncryptionError),
    #[error("Merge failed: {0}")]
    Merge(#[from] crate::util::helm::MergeError),
    #[error("Merge failed: {0}")]
    MergeFailed(String), // Placeholder if needed
}

#[derive(Error, Debug)]
pub enum EncryptionError {
    #[error("sops executable not found. Please install sops to decrypt secrets.")]
    SopsNotFound,
    #[error("Failed to execute sops: {0}")]
    SopsExecutionFailed(#[source] std::io::Error),
    #[error("Failed to decrypt '{0}': {1}")]
    DecryptionFailed(String, String),
    #[error("Decrypted content is not valid UTF-8: {0}")]
    Utf8Error(#[from] std::string::FromUtf8Error),
}

#[derive(Error, Debug)]
pub enum DagError {
    #[error("Unknown dependency: '{1}' for chart '{0}'")]
    UnknownDependency(String, String),
    #[error("Circular dependency detected involving chart '{0}'")]
    CircularDependency(String),
    #[error("Cycle detected: {0:?}")]
    CycleDetected(daggy::petgraph::algo::Cycle<daggy::NodeIndex>),
    #[error("Chart '{0}' not found in graph")]
    ChartNotFound(String),
    #[error("Internal error: Node weight missing")]
    NodeWeightMissing,
    #[error("Internal error: {0}")]
    InternalError(String),
}

#[derive(Error, Debug)]
pub enum FilterError {
    #[error("The following specified charts do not exist: {0}")]
    ChartsNotFound(String),
}

#[derive(Error, Debug)]
pub enum ProgressError {
    #[error("Template error: {0}")]
    TemplateError(String),
}

#[derive(Error, Debug)]
pub enum InteractionError {
    #[error("Interaction failed: {0}")]
    Dialoguer(#[from] dialoguer::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Formats an error for display.
pub fn format_error(err: &(dyn std::error::Error + 'static)) -> String {
    // Try to find ValidationErrors in the source chain
    let mut current = Some(err);
    while let Some(cause) = current {
        if let Some(validation_errors) = cause.downcast_ref::<ValidationErrors>() {
            return format_validation_errors(validation_errors);
        }
        current = cause.source();
    }

    // Default formatting
    let mut output = format!("{} {}", style("Error:").bold().red(), err);

    let mut cause = err.source();
    while let Some(c) = cause {
        output.push_str(&format!(
            "\n  {} {}",
            style("Caused by:").bold().yellow(),
            c
        ));
        cause = c.source();
    }

    output
}

fn format_validation_errors(errors: &ValidationErrors) -> String {
    let mut output = String::new();
    output.push_str(&format!(
        "{} Configuration validation failed\n\n",
        style("Error:").bold().red()
    ));

    let mut issues = Vec::new();
    flatten_errors(errors, &mut issues, None);

    // Sort issues for consistent output
    issues.sort();

    for issue in issues {
        output.push_str(&format!("{}\n", issue));
    }

    output
}

fn flatten_errors(
    errors: &ValidationErrors,
    issues: &mut Vec<String>,
    parent_field: Option<String>,
) {
    for (field, error_kind) in errors.errors() {
        match error_kind {
            ValidationErrorsKind::Field(field_errors) => {
                for error in field_errors {
                    let field_label = parent_field
                        .clone()
                        .map(|p| format!("{}.{}", p, field))
                        .unwrap_or_else(|| field.to_string());

                    let message = format_field_error(&field_label, error);
                    issues.push(message);
                }
            }
            ValidationErrorsKind::Struct(struct_errors) => {
                let prefix = parent_field
                    .clone()
                    .map(|p| format!("{}.{}", p, field))
                    .unwrap_or_else(|| field.to_string());
                flatten_errors(struct_errors, issues, Some(prefix));
            }
            ValidationErrorsKind::List(list_errors) => {
                for (index, struct_errors) in list_errors {
                    let prefix = parent_field
                        .clone()
                        .map(|p| format!("{}.{}[{}]", p, field, index))
                        .unwrap_or_else(|| format!("{}[{}]", field, index));
                    flatten_errors(struct_errors, issues, Some(prefix));
                }
            }
        }
    }
}

fn format_field_error(field: &str, error: &validator::ValidationError) -> String {
    match error.code.as_ref() {
        "duplicate_chart_name_namespace" => {
            let name = error
                .params
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            let namespace = error
                .params
                .get("namespace")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            format!(
                "  - {}: Duplicate chart detected: name='{}', namespace='{}'",
                style("Error").red(),
                style(name).yellow(),
                style(namespace).yellow()
            )
        }
        "duplicate_repository_names" => {
            format!(
                "  - {}: Duplicate repository names found.",
                style("Error").red()
            )
        }
        "duplicate_destination_names" => {
            format!(
                "  - {}: Duplicate destination names found.",
                style("Error").red()
            )
        }
        "chart_repo_not_found" => {
            let chart_name = error
                .params
                .get("chart_name")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            let repo_name = error
                .params
                .get("repo_name")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            format!(
                "  - {}: Chart '{}' references a repository '{}' that does not exist.",
                style("Error").red(),
                style(chart_name).yellow(),
                style(repo_name).yellow()
            )
        }

        "chart_dest_not_found" => {
            format!(
                "  - {}: Chart references a destination that does not exist.",
                style("Error").red()
            )
        }
        "values_file_not_found" => {
            let file = error
                .params
                .get("file")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            format!(
                "  - {}: Values file not found: '{}'",
                style("Error").red(),
                style(file).yellow()
            )
        }
        "variable_file_not_found" => {
            let file = error
                .params
                .get("file")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            format!(
                "  - {}: Variable file not found: '{}'",
                style("Error").red(),
                style(file).yellow()
            )
        }
        "secrets_file_not_found" => {
            let file = error
                .params
                .get("file")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            format!(
                "  - {}: Secrets file not found: '{}'",
                style("Error").red(),
                style(file).yellow()
            )
        }
        _ => {
            let message = if let Some(msg) = &error.message {
                msg.to_string()
            } else {
                error.code.to_string()
            };
            format!(
                "  - {}: Field '{}' failed validation: {}",
                style("Error").red(),
                field, // fixed variable name from 'field_label' which wasn't in scope in original snippet, assuming 'field' arg
                message
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use validator::{ValidationError, ValidationErrors};

    #[test]
    fn test_format_validation_errors() {
        let mut errors = ValidationErrors::new();
        let mut err = ValidationError::new("custom_code");
        err.message = Some(std::borrow::Cow::from("Custom message"));
        errors.add("field1", err);

        // format_error expects &dyn std::error::Error
        // ValidationErrors implements Error
        let formatted = format_error(&errors);

        assert!(formatted.contains("Configuration validation failed"));
        assert!(formatted.contains("field1"));
        assert!(formatted.contains("Custom message"));
    }
}
