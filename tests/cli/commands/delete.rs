use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use vesshelm::config::{Chart, Config, Destination};

#[test]
fn test_delete_non_existent_chart() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config_path = temp_dir.path().join("vesshelm.yaml");

    // Create minimal config
    let config = Config {
        variable_files: None,
        repositories: vec![],
        charts: vec![],
        destinations: vec![],
        vesshelm: None,
    };
    let yaml = serde_yaml_ng::to_string(&config).unwrap();
    fs::write(&config_path, yaml).unwrap();

    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("vesshelm"));

    cmd.current_dir(&temp_dir)
        .arg("--config")
        .arg(config_path.to_str().unwrap())
        .arg("delete")
        .arg("non-existent-chart")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "Chart 'non-existent-chart' not found",
        ));
}

#[test]
fn test_delete_chart_no_interactive() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config_path = temp_dir.path().join("vesshelm.yaml");

    // Setup: Create a config with one chart
    let chart = Chart {
        name: "test-chart".to_string(),
        repo_name: Some("test-repo".to_string()),
        version: Some("1.0.0".to_string()),
        namespace: "default".to_string(),
        dest: None,
        chart_path: None,
        no_sync: false,
        no_deploy: false,
        comment: None,
        values_files: None,
        helm_args_append: None,
        helm_args_override: None,
        values: None,
        depends: None,
    };

    let config = Config {
        repositories: vec![vesshelm::config::Repository {
            name: "test-repo".to_string(),
            url: "https://example.com".to_string(),
            r#type: Default::default(),
        }],
        charts: vec![chart],
        destinations: vec![Destination {
            name: "default".to_string(),
            path: "charts".to_string(),
        }],
        vesshelm: None,
        variable_files: None,
    };

    let yaml = serde_yaml_ng::to_string(&config).unwrap();
    fs::write(&config_path, yaml).unwrap();

    // Create a fake chart directory to verify deletion
    let chart_dir = temp_dir.path().join("charts/test-chart");
    fs::create_dir_all(&chart_dir).unwrap();
    fs::write(chart_dir.join("Chart.yaml"), "name: test-chart").unwrap();

    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("vesshelm"));

    cmd.current_dir(&temp_dir)
        .arg("--config")
        .arg(config_path.to_str().unwrap())
        .arg("delete")
        .arg("test-chart")
        .arg("--no-interactive")
        .assert()
        .success()
        .stdout(predicate::str::contains("Deletion completed"));

    // Verify config update
    let updated_content = fs::read_to_string(&config_path).unwrap();
    assert!(!updated_content.contains("test-chart"));

    // Verify directory deletion
    assert!(!chart_dir.exists());
}
