use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;

#[test]
fn test_validate_missing_file() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("vesshelm"));
    cmd.arg("validate").arg("--config").arg("nonexistent.yaml");
    cmd.assert().failure().stderr(predicate::str::contains(
        "Failed to read configuration file",
    ));
}

#[test]
fn test_validate_missing_values_file() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config_path = temp_dir.path().join("vesshelm.yaml");
    let config_content = r#"
repositories: []
charts:
  - name: my-chart
    namespace: default
    values_files:
      - missing-values.yaml
destinations:
  - name: default
    path: ./charts
"#;
    fs::write(&config_path, config_content).unwrap();

    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("vesshelm"));
    cmd.current_dir(&temp_dir)
        .arg("validate")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Values file not found"));
}
