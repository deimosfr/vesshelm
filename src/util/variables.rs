use crate::util::encryption;
use crate::util::helm::merge_yaml_values;
use anyhow::{Context, Result};
use serde_yaml_ng::Value;
use std::path::Path;

pub fn load_variables(paths: &[String], base_path: &Path) -> Result<Value> {
    let mut merged_context = Value::Null;

    for path_str in paths {
        let path = base_path.join(path_str);
        if path.exists() {
            // 1. Read (and transparently decrypt)
            let content = read_file_handling_encryption(&path)?;

            // 2. Render content using current context
            // Use filename as template name for good error messages
            let template_name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("values");

            let rendered_content = render_template(&content, &merged_context, template_name)?;

            // 3. Parse YAML
            let value: Value = serde_yaml_ng::from_str(&rendered_content)
                .with_context(|| format!("Failed to parse variable file: {:?}", path))?;

            // 4. Merge into context
            if merged_context.is_null() {
                merged_context = value;
            } else {
                merge_yaml_values(&mut merged_context, &value)?;
            }
        } else {
            anyhow::bail!("Variable file not found: {:?}", path);
        }
    }

    Ok(merged_context)
}

pub fn render_values_file(path: &Path, context: &Value) -> Result<String> {
    let content = read_file_handling_encryption(path)?;
    let template_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("values");

    render_template(&content, context, template_name)
}

fn render_template(content: &str, context: &Value, template_name: &str) -> Result<String> {
    let mut env = minijinja::Environment::new();
    env.set_undefined_behavior(minijinja::UndefinedBehavior::Strict);
    env.add_template(template_name, content)?;

    let tmpl = env.get_template(template_name)?;
    let rendered = tmpl.render(context)?;

    Ok(rendered)
}

fn read_file_handling_encryption(path: &Path) -> Result<String> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read file: {:?}", path))?;

    if encryption::is_sops_encrypted(&content) {
        encryption::decrypt_sops_file(path)
    } else {
        Ok(content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_render_values_file() -> Result<()> {
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

        let rendered = render_values_file(path, &context)?;

        assert!(rendered.contains("region: us-east-1"));
        assert!(rendered.contains("cluster: wandering-star"));

        Ok(())
    }

    #[test]
    fn test_load_variables() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let path1 = dir.path().join("vars1.yaml");
        let path2 = dir.path().join("vars2.yaml");

        std::fs::write(&path1, "key1: value1\ncommon: old")?;
        std::fs::write(&path2, "key2: value2\ncommon: new")?;

        let paths = vec!["vars1.yaml".to_string(), "vars2.yaml".to_string()];

        let loaded = load_variables(&paths, dir.path())?;

        let mapping = loaded.as_mapping().unwrap();
        assert_eq!(
            mapping.get(&Value::from("key1")),
            Some(&Value::from("value1"))
        );
        assert_eq!(
            mapping.get(&Value::from("key2")),
            Some(&Value::from("value2"))
        );
        assert_eq!(
            mapping.get(&Value::from("common")),
            Some(&Value::from("new"))
        );

        Ok(())
    }

    #[test]
    fn test_render_values_file_missing_var() -> Result<()> {
        let mut file = NamedTempFile::new()?;
        write!(file, "region: {{{{ region }}}}")?;
        file.flush()?;

        let path = file.path();
        // Context empty
        let context: Value = serde_yaml_ng::from_str("{}")?;

        let result = render_values_file(path, &context);

        assert!(result.is_err());
        let err = result.unwrap_err();
        let err_str = format!("{:#}", err);
        // Minijinja strict undefined behavior reports "undefined value"
        assert!(err_str.contains("undefined value"));

        Ok(())
    }

    #[test]
    fn test_load_variables_interpolation() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let path1 = dir.path().join("vars1.yaml");
        let path2 = dir.path().join("vars2.yaml");

        std::fs::write(&path1, "env: prod")?;
        // This var depends on 'env' from vars1.yaml
        std::fs::write(&path2, "fqdn: \"app-{{ env }}.example.com\"")?;

        let paths = vec!["vars1.yaml".to_string(), "vars2.yaml".to_string()];

        let loaded = load_variables(&paths, dir.path())?;
        let mapping = loaded.as_mapping().unwrap();

        // Check if fqdn is interpolated
        let fqdn = mapping.get(&Value::from("fqdn")).unwrap().as_str().unwrap();

        // If interpolation works, it should be "app-prod.example.com"
        // If it doesn't, it will be "app-{{ env }}.example.com"
        println!("Resolved FQDN: {}", fqdn);

        // We expect this to FAIL if interpolation is not implemented
        assert_eq!(fqdn, "app-prod.example.com");

        Ok(())
    }
}
