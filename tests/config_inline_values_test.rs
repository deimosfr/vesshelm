use anyhow::Result;
use std::fs;
use tempfile::TempDir;
use vesshelm::config::Config;

#[test]
fn test_config_inline_values_map_and_sequence() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let config_path = temp_dir.path().join("vesshelm.yaml");

    // Test Case 1: Values as Map
    let config_map = r#"
repositories:
  - name: stable
    url: https://charts.helm.sh/stable
charts:
  - name: chart-map
    repo_name: stable
    namespace: default
    values:
      key: value
destinations:
  - name: default
    path: ./charts
"#;
    fs::write(&config_path, config_map)?;
    let config = Config::load_from_path(&config_path)?;
    assert!(config.charts[0].values.is_some());
    assert!(config.charts[0].values.as_ref().unwrap().is_mapping());

    // Test Case 2: Values as Sequence
    let config_seq = r#"
repositories:
  - name: stable
    url: https://charts.helm.sh/stable
charts:
  - name: chart-seq
    repo_name: stable
    namespace: default
    values:
      - key: value
destinations:
  - name: default
    path: ./charts
"#;
    fs::write(&config_path, config_seq)?;
    let config = Config::load_from_path(&config_path)?;
    assert!(config.charts[0].values.is_some());
    assert!(config.charts[0].values.as_ref().unwrap().is_sequence());

    Ok(())
}

#[test]
fn test_config_inline_values_invalid_type() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let config_path = temp_dir.path().join("vesshelm.yaml");

    // Test Case 3: Values as String (Invalid)
    let config_invalid = r#"
repositories:
  - name: stable
    url: https://charts.helm.sh/stable
charts:
  - name: chart-invalid
    repo_name: stable
    namespace: default
    values: "invalid"
destinations:
  - name: default
    path: ./charts
"#;
    fs::write(&config_path, config_invalid)?;
    let result = Config::load_from_path(&config_path);
    assert!(result.is_err());
    // Error could be from serde deserialization OR our custom validation
    let err = result.unwrap_err();
    let err_str = format!("{}", err);
    // Since we changed the type to Option<Value>, serde accepts any valid YAML value.
    // So our validation logic in validate_config should catch it.
    assert!(
        err_str.contains("values_invalid_type") || err_str.contains("Validation failed"),
        "Unexpected error: {}",
        err_str
    );

    Ok(())
}
