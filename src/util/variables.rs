use crate::util::helm::merge_values;
use anyhow::{Context, Result};
use serde_yaml_ng::Value;
use std::path::Path;

pub fn load_variables(paths: &[String], base_path: &Path) -> Result<Value> {
    let mut loaded_values = Vec::new();

    for path_str in paths {
        let path = base_path.join(path_str);
        if path.exists() {
            let content = std::fs::read_to_string(&path)
                .with_context(|| format!("Failed to read variable file: {:?}", path))?;
            let value: Value = serde_yaml_ng::from_str(&content)
                .with_context(|| format!("Failed to parse variable file: {:?}", path))?;
            loaded_values.push(value);
        } else {
            anyhow::bail!("Variable file not found: {:?}", path);
        }
    }

    if loaded_values.is_empty() {
        return Ok(Value::Null);
    }

    // Re-implementing a simple merge here returning Value or refactoring helm::merge_values might be best.
    // For now let's use the one in helm.rs but it returns string.

    let merged_yaml = merge_values(&loaded_values)?;
    let merged_value: Value = serde_yaml_ng::from_str(&merged_yaml)?;

    Ok(merged_value)
}

pub fn render_values_file(path: &Path, context: &Value) -> Result<String> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read values file: {:?}", path))?;

    let mut env = minijinja::Environment::new();
    env.set_undefined_behavior(minijinja::UndefinedBehavior::Strict);
    // Add default filters if needed, but minijinja has built-ins.
    // We register the template string.
    env.add_template("values", &content)?;

    let tmpl = env.get_template("values")?;
    let rendered = tmpl.render(context)?;

    Ok(rendered)
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
}
