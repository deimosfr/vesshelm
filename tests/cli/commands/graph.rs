use assert_cmd::Command;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_graph_command() {
    let temp = TempDir::new().unwrap();
    let config_path = temp.path().join("vesshelm.yaml");

    let config_content = r#"
charts:
  - name: chart-a
    repo_name: stable
    version: 1.0.0
    namespace: default
    depends:
      - chart-b

  - name: chart-b
    repo_name: stable
    version: 1.0.0
    namespace: default

repositories:
  - name: stable
    url: http://localhost

destinations:
  - name: default
    path: ./charts
"#;

    fs::write(&config_path, config_content).unwrap();

    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("vesshelm"));
    cmd.current_dir(temp.path())
        .arg("graph")
        .assert()
        .success()
        .stdout(predicates::str::contains("chart-a"))
        .stdout(predicates::str::contains("chart-b"));
}
