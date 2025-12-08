use crate::util::encryption;
use crate::util::error::VariablesError;
use crate::util::helm::merge_yaml_values;
use serde_yaml_ng::Value;
use std::path::Path;

pub async fn load_variables(paths: &[String], base_path: &Path) -> Result<Value, VariablesError> {
    let mut merged_context = Value::Null;

    for path_str in paths {
        let path = base_path.join(path_str);
        if path.exists() {
            // 1. Read (and transparently decrypt)
            let content = read_file_handling_encryption(&path).await?;

            // 2. Render content using current context
            // Use filename as template name for good error messages
            let template_name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("values");

            let rendered_content = render_template(&content, &merged_context, template_name)?;

            // 3. Parse YAML
            let value: Value = serde_yaml_ng::from_str(&rendered_content).map_err(|e| {
                VariablesError::ParseYamlFailed(path.to_string_lossy().to_string(), e)
            })?;

            // 4. Merge into context
            if merged_context.is_null() {
                merged_context = value;
            } else {
                merge_yaml_values(&mut merged_context, &value)?;
            }
        } else {
            return Err(VariablesError::FileNotFound(
                path.to_string_lossy().to_string(),
            ));
        }
    }

    Ok(merged_context)
}

pub async fn render_values_file(path: &Path, context: &Value) -> Result<String, VariablesError> {
    let content = read_file_handling_encryption(path).await?;
    let template_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("values");

    render_template(&content, context, template_name)
}

pub fn render_string(content: &str, context: &Value) -> Result<String, VariablesError> {
    render_template(content, context, "inline-values")
}

fn render_template(
    content: &str,
    context: &Value,
    template_name: &str,
) -> Result<String, VariablesError> {
    let mut env = minijinja::Environment::new();
    env.set_undefined_behavior(minijinja::UndefinedBehavior::Strict);
    // Remove conflicting builtins by shadowing them with Undefined
    env.add_global("namespace", minijinja::value::Value::UNDEFINED);

    // Pre-process content to ignore commented out lines which might contain undefined variables
    let processed_content = preprocess_template(content);

    env.add_template(template_name, &processed_content)
        .map_err(|e| VariablesError::TemplateRenderFailed(template_name.to_string(), e))?;

    let tmpl = env
        .get_template(template_name)
        .map_err(|e| VariablesError::TemplateRenderFailed(template_name.to_string(), e))?;
    let rendered = tmpl
        .render(minijinja::value::Serde(context))
        .map_err(|e| VariablesError::TemplateRenderFailed(template_name.to_string(), e))?;

    Ok(rendered)
}

fn preprocess_template(content: &str) -> String {
    content
        .lines()
        .map(|line| {
            if line.trim_start().starts_with('#') {
                ""
            } else {
                line
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

async fn read_file_handling_encryption(path: &Path) -> Result<String, VariablesError> {
    let content = std::fs::read_to_string(path).map_err(VariablesError::Io)?;

    if encryption::is_sops_encrypted(&content) {
        Ok(encryption::decrypt_sops_file(path).await?)
    } else {
        Ok(content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

    #[tokio::test]
    async fn test_render_values_file() -> Result<()> {
        let mut file = NamedTempFile::new()?;
        write!(
            file,
            "region: {{{{ region }}}}\ncluster: {{{{ cluster.name }}}}"
        )?;

        let path = file.path();
        let context_str = r#"
region: us-east-1
cluster:
  name: wandering-star
"#;
        let context: Value = serde_yaml_ng::from_str(context_str)?;

        let rendered = render_values_file(path, &context).await?;

        assert!(rendered.contains("region: us-east-1"));
        assert!(rendered.contains("cluster: wandering-star"));

        Ok(())
    }

    #[tokio::test]
    async fn test_load_variables() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let path1 = dir.path().join("vars1.yaml");
        let path2 = dir.path().join("vars2.yaml");

        std::fs::write(&path1, "key1: value1\ncommon: old")?;
        std::fs::write(&path2, "key2: value2\ncommon: new")?;

        let paths = vec!["vars1.yaml".to_string(), "vars2.yaml".to_string()];

        let loaded = load_variables(&paths, dir.path()).await?;

        let mapping = loaded.as_mapping().unwrap();
        assert_eq!(
            mapping.get(Value::from("key1")),
            Some(&Value::from("value1"))
        );
        assert_eq!(
            mapping.get(Value::from("key2")),
            Some(&Value::from("value2"))
        );
        assert_eq!(
            mapping.get(Value::from("common")),
            Some(&Value::from("new"))
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_render_values_file_missing_var() -> Result<()> {
        let mut file = NamedTempFile::new()?;
        write!(file, "region: {{{{ region }}}}")?;
        file.flush()?;

        let path = file.path();
        // Context empty
        let context: Value = serde_yaml_ng::from_str("{}")?;

        let result = render_values_file(path, &context).await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        let err_str = format!("{:#}", err);
        // Minijinja strict undefined behavior reports "undefined value"
        assert!(err_str.contains("undefined value"));

        Ok(())
    }

    #[tokio::test]
    async fn test_load_variables_interpolation() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let path1 = dir.path().join("vars1.yaml");
        let path2 = dir.path().join("vars2.yaml");

        std::fs::write(&path1, "env: prod")?;
        // This var depends on 'env' from vars1.yaml
        std::fs::write(&path2, "fqdn: \"app-{{ env }}.example.com\"")?;

        let paths = vec!["vars1.yaml".to_string(), "vars2.yaml".to_string()];

        let loaded = load_variables(&paths, dir.path()).await?;
        let mapping = loaded.as_mapping().unwrap();

        // Check if fqdn is interpolated
        let fqdn = mapping.get(Value::from("fqdn")).unwrap().as_str().unwrap();

        // If interpolation works, it should be "app-prod.example.com"
        // If it doesn't, it will be "app-{{ env }}.example.com"
        println!("Resolved FQDN: {}", fqdn);

        // We expect this to FAIL if interpolation is not implemented
        assert_eq!(fqdn, "app-prod.example.com");

        Ok(())
    }

    #[test]
    fn test_render_string() -> Result<()> {
        let context_str = "env: prod";
        let context: Value = serde_yaml_ng::from_str(context_str)?;

        let content = "key: {{ env }}";
        let rendered = render_string(content, &context)?;

        assert_eq!(rendered, "key: prod");
        Ok(())
    }

    #[test]
    fn test_render_namespace_conflict() -> Result<()> {
        // User scenario: namespace is NOT in context, but they expect empty or error, NOT the function object
        // Or if it IS in context, it should win.

        // Case 1: In context
        let context_str = "namespace: my-namespace";
        let context: Value = serde_yaml_ng::from_str(context_str)?;
        let content = "ns: {{ namespace }}";
        let rendered = render_string(content, &context)?;
        assert_eq!(rendered, "ns: my-namespace");

        // Case 2: Not in context (Should fail or be empty, but definitely not function)
        let context_empty: Value = serde_yaml_ng::from_str("{}")?;
        // If strict is on, this should fail.
        let result = render_string(content, &context_empty);
        // We want to see what happens.
        if let Ok(res) = result {
            // If this prints "ns: <function namespace>" that's the bug.
            println!("Rendered without context: {}", res);
            assert_ne!(res, "ns: minijinja::functions::builtins::namespace");
        }

        Ok(())
    }

    #[test]
    fn test_render_ignore_comments() -> Result<()> {
        let context_str = "env: prod";
        let context: Value = serde_yaml_ng::from_str(context_str)?;

        let content = r#"
# This is a comment with {{ undefined_var }}
key: {{ env }}
  # Another comment with {{ another_undefined }}
"#;
        let rendered = render_string(content, &context)?;

        // The comments should be empty lines, and the key should be interpolated
        assert!(rendered.contains("key: prod"));
        assert!(!rendered.contains("undefined_var"));

        Ok(())
    }

    #[test]
    fn test_render_line_numbers_preserved() -> Result<()> {
        let context_empty: Value = serde_yaml_ng::from_str("{}")?;

        // We want an error at line 3
        let content = "line 1\n# comment\nkey: {{ undefined_var }}";

        let result = render_string(content, &context_empty);
        assert!(result.is_err());

        let err_str = format!("{:#}", result.unwrap_err());
        // Jinja (via minijinja) usually reports line numbers in error messages.
        // We want to see ':3' reflecting the template location.
        assert!(
            err_str.contains(":3"),
            "Error message should point to line 3, got: {}",
            err_str
        );

        Ok(())
    }
}
