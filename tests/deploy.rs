use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use tempfile::TempDir;

fn setup_mock_helm(temp_dir: &TempDir) {
    let mock_helm_path = temp_dir.path().join("helm");
    let mock_helm_content = r#"#!/bin/sh
echo "Mock Helm called with: $@"
if [ "$HELM_DIFF_COLOR" = "true" ]; then
    echo "Mock Helm Diff Color is set"
fi
"#;
    fs::write(&mock_helm_path, mock_helm_content).unwrap();
    let mut perms = fs::metadata(&mock_helm_path).unwrap().permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&mock_helm_path, perms).unwrap();
}

#[test]
fn test_deploy_dry_run() {
    let temp = TempDir::new().unwrap();
    setup_mock_helm(&temp);

    let config_path = temp.path().join("vesshelm.yaml");
    let config_content = r#"
repositories:
  - name: stable
    url: https://charts.helm.sh/stable

charts:
  - name: chart-a
    repo_name: stable
    version: 1.0.0
    namespace: default

destinations:
  - name: default
    path: ./charts

helm:
    args: "upgrade --install {{ name }} {{ destination }} -n {{ namespace }}"
"#;
    fs::write(&config_path, config_content).unwrap();

    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("vesshelm"));
    cmd.current_dir(temp.path())
        .env("PATH", temp.path()) // Override PATH to pick up mock helm
        .arg("deploy")
        .arg("--dry-run")
        .arg("--no-interactive")
        .arg("--no-progress")
        .assert()
        .success()
        .stdout(predicates::str::contains(
            "Mock Helm called with: diff upgrade",
        ))
        .stdout(predicates::str::contains("Mock Helm Diff Color is set"));
}

#[test]
fn test_deploy_full() {
    let temp = TempDir::new().unwrap();
    setup_mock_helm(&temp);

    let config_path = temp.path().join("vesshelm.yaml");
    let config_content = r#"
repositories:
  - name: stable
    url: https://charts.helm.sh/stable

charts:
  - name: chart-a
    repo_name: stable
    version: 1.0.0
    namespace: default

destinations:
  - name: default
    path: ./charts

helm:
    args: "upgrade --install {{ name }} {{ destination }} -n {{ namespace }}"
"#;
    fs::write(&config_path, config_content).unwrap();

    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("vesshelm"));
    cmd.current_dir(temp.path())
        .env("PATH", temp.path())
        .arg("deploy")
        .arg("--no-interactive")
        .arg("--no-progress")
        .assert()
        .success()
        .stdout(predicates::str::contains(
            "Mock Helm called with: upgrade --install chart-a",
        ))
        .stdout(predicates::str::contains("Deployment ended"));
}

#[test]
fn test_deploy_filter() {
    let temp = TempDir::new().unwrap();
    setup_mock_helm(&temp);

    let config_path = temp.path().join("vesshelm.yaml");
    let config_content = r#"
repositories:
  - name: stable
    url: https://charts.helm.sh/stable

charts:
  - name: chart-a
    repo_name: stable
    version: 1.0.0
    namespace: default
  - name: chart-b
    repo_name: stable
    version: 1.0.0
    namespace: default

destinations:
  - name: default
    path: ./charts

helm:
    args: "upgrade --install {{ name }} {{ destination }} -n {{ namespace }}"
"#;
    fs::write(&config_path, config_content).unwrap();

    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("vesshelm"));
    cmd.current_dir(temp.path())
        .env("PATH", temp.path())
        .arg("deploy")
        .arg("--no-interactive")
        .arg("--no-progress")
        .arg("--only")
        .arg("chart-b")
        .assert()
        .success()
        .stdout(predicates::str::contains(
            "Mock Helm called with: upgrade --install chart-b",
        ))
        .stdout(
            predicates::str::contains("Mock Helm called with: upgrade --install chart-a").not(),
        );
}

fn setup_mock_helm_empty_diff(temp_dir: &TempDir) {
    let mock_helm_path = temp_dir.path().join("helm");
    // Script that outputs nothing if first arg is "diff", otherwise echoes args
    let mock_helm_content = r#"#!/bin/sh
if [ "$1" = "diff" ]; then
    exit 0
fi
echo "Mock Helm called with: $@"
"#;
    fs::write(&mock_helm_path, mock_helm_content).unwrap();
    let mut perms = fs::metadata(&mock_helm_path).unwrap().permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&mock_helm_path, perms).unwrap();
}

#[test]
fn test_deploy_skip_no_changes() {
    let temp = TempDir::new().unwrap();
    setup_mock_helm_empty_diff(&temp);

    let config_path = temp.path().join("vesshelm.yaml");
    let config_content = r#"
repositories:
  - name: stable
    url: https://charts.helm.sh/stable

charts:
  - name: chart-a
    repo_name: stable
    version: 1.0.0
    namespace: default

destinations:
  - name: default
    path: ./charts

helm:
    args: "upgrade --install {{ name }} {{ destination }} -n {{ namespace }}"

"#;
    fs::write(&config_path, config_content).unwrap();

    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("vesshelm"));
    cmd.current_dir(temp.path())
        .env("PATH", temp.path())
        .arg("deploy")
        .arg("--no-interactive")
        .arg("--no-progress")
        .assert()
        .success()
        .stdout(predicates::str::contains(
            "No changes for chart-a. Skipping.",
        ))
        .stdout(predicates::str::contains("Skipped:  1"))
        .stdout(predicates::str::contains("Mock Helm called with: upgrade").not());
}

#[test]
fn test_deploy_force_no_changes() {
    let temp = TempDir::new().unwrap();
    setup_mock_helm_empty_diff(&temp);

    let config_path = temp.path().join("vesshelm.yaml");
    let config_content = r#"
repositories:
  - name: stable
    url: https://charts.helm.sh/stable

charts:
  - name: chart-a
    repo_name: stable
    version: 1.0.0
    namespace: default

destinations:
  - name: default
    path: ./charts

helm:
    args: "upgrade --install {{ name }} {{ destination }} -n {{ namespace }}"

"#;
    fs::write(&config_path, config_content).unwrap();

    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("vesshelm"));
    cmd.current_dir(temp.path())
        .env("PATH", temp.path())
        .arg("deploy")
        .arg("--no-interactive")
        .arg("--no-progress")
        .arg("--force")
        .assert()
        .success()
        .stdout(predicates::str::contains(
            "No changes detected, but forcing deployment for chart-a",
        ))
        .stdout(predicates::str::contains("Mock Helm called with: upgrade"))
        .stdout(predicates::str::contains("Deployed: 1"));
}

#[test]
fn test_deploy_force_respects_no_deploy() {
    let temp = TempDir::new().unwrap();
    setup_mock_helm_empty_diff(&temp);

    let config_path = temp.path().join("vesshelm.yaml");
    let config_content = r#"
repositories:
  - name: stable
    url: https://charts.helm.sh/stable

charts:
  - name: chart-a
    repo_name: stable
    version: 1.0.0
    namespace: default
    no_deploy: true

destinations:
  - name: default
    path: ./charts

helm:
    args: "upgrade --install {{ name }} {{ destination }} -n {{ namespace }}"

"#;
    fs::write(&config_path, config_content).unwrap();

    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("vesshelm"));
    cmd.current_dir(temp.path())
        .env("PATH", temp.path())
        .arg("deploy")
        .arg("--no-interactive")
        .arg("--no-progress")
        .arg("--force")
        .assert()
        .success()
        .stdout(predicates::str::contains("(no_deploy=true)"))
        .stdout(predicates::str::contains("Skipped:  1"))
        .stdout(predicates::str::contains("Mock Helm called with: upgrade").not());
}

#[test]
fn test_deploy_force_conflict_dry_run() {
    let temp = TempDir::new().unwrap();
    // No need to setup mock helm mostly, as clap check happens before execution

    let config_path = temp.path().join("vesshelm.yaml");
    let config_content = r#"
repositories:
  - name: stable
    url: https://charts.helm.sh/stable

charts:
  - name: chart-a
    repo_name: stable
    version: 1.0.0
    namespace: default

destinations:
  - name: default
    path: ./charts
"#;
    fs::write(&config_path, config_content).unwrap();

    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("vesshelm"));
    cmd.current_dir(temp.path())
        .env("PATH", temp.path())
        .arg("deploy")
        .arg("--force")
        .arg("--dry-run")
        .assert()
        .failure()
        .stderr(predicates::str::contains(
            "argument '--force' cannot be used with '--dry-run'",
        ));
}
