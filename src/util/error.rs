use anyhow::Error;
use colored::Colorize;
use validator::{ValidationErrors, ValidationErrorsKind};

/// Formats an error for display.
///
/// If the error is a `ValidationErrors` (possibly wrapped), it will be formatted as a list of issues.
/// Otherwise, it will be formatted as a simple error message.
pub fn format_error(err: &Error) -> String {
    // Try to find ValidationErrors in the chain
    for cause in err.chain() {
        if let Some(validation_errors) = cause.downcast_ref::<ValidationErrors>() {
            return format_validation_errors(validation_errors);
        }
    }

    // Default formatting
    format!("{} {}", "Error:".bold().red(), err)
}

fn format_validation_errors(errors: &ValidationErrors) -> String {
    let mut output = String::new();
    output.push_str(&format!(
        "{} Configuration validation failed\n\n",
        "Error:".bold().red()
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
                    // Try to make the field name more readable if appropriate,
                    // but for top-level custom formatting we might ignore it.
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
                "Error".red(),
                name.yellow(),
                namespace.yellow()
            )
        }
        "duplicate_repository_names" => {
            format!("  - {}: Duplicate repository names found.", "Error".red())
        }
        "duplicate_destination_names" => {
            format!("  - {}: Duplicate destination names found.", "Error".red())
        }
        "chart_repo_not_found" => {
            format!(
                "  - {}: Chart references a repository that does not exist.",
                "Error".red()
            )
        }
        "chart_dest_not_found" => {
            format!(
                "  - {}: Chart references a destination that does not exist.",
                "Error".red()
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
                "Error".red(),
                file.yellow()
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
                "Error".red(),
                field,
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
    fn test_format_error_simple() {
        let err = Error::msg("Simple error");
        let formatted = format_error(&err);
        assert!(formatted.contains("Simple error"));
        assert!(formatted.contains("Error:"));
    }

    #[test]
    fn test_format_validation_errors() {
        let mut errors = ValidationErrors::new();
        let mut err = ValidationError::new("custom_code");
        err.message = Some(std::borrow::Cow::from("Custom message"));
        errors.add("field1", err);

        let anyhow_err = Error::from(errors);
        let formatted = format_error(&anyhow_err);

        assert!(formatted.contains("Configuration validation failed"));
        assert!(formatted.contains("field1"));
        assert!(formatted.contains("Custom message"));
    }

    #[test]
    fn test_format_field_error_known_codes() {
        let known_codes = vec![
            ("duplicate_chart_name_namespace", "Duplicate chart detected"),
            ("duplicate_repository_names", "Duplicate repository names"),
            ("duplicate_destination_names", "Duplicate destination names"),
            (
                "chart_repo_not_found",
                "Chart references a repository that does not exist",
            ),
            (
                "chart_dest_not_found",
                "Chart references a destination that does not exist",
            ),
            ("values_file_not_found", "Values file not found"),
        ];

        for (code, expected_msg) in known_codes {
            let mut errors = ValidationErrors::new();
            let mut err = ValidationError::new(code);
            err.add_param(std::borrow::Cow::from("name"), &"test-chart");
            err.add_param(std::borrow::Cow::from("namespace"), &"default");
            err.add_param(std::borrow::Cow::from("file"), &"values.yaml");

            errors.add("test_field", err);

            let anyhow_err = Error::from(errors);
            let formatted = format_error(&anyhow_err);

            assert!(
                formatted.contains(expected_msg),
                "Failed for code: {}",
                code
            );
        }
    }

    #[test]
    fn test_format_validation_errors_nested() {
        use validator::Validate;

        #[derive(Validate)]
        struct Inner {
            #[validate(length(min = 5, message = "Inner too short"))]
            val: String,
        }

        #[derive(Validate)]
        struct Outer {
            #[validate(nested)]
            inner: Inner,
        }

        let outer = Outer {
            inner: Inner {
                val: "bad".to_string(),
            },
        };

        let res = outer.validate();
        assert!(res.is_err());
        let errors = res.err().unwrap();

        let anyhow_err = Error::from(errors);
        let formatted = format_error(&anyhow_err);

        assert!(formatted.contains("inner.val"));
        assert!(formatted.contains("Inner too short"));
    }
}
