use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use tempfile::TempDir;

fn setup_mock_helm_initial(temp_dir: &TempDir) {
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
    setup_mock_helm_initial(&temp);

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

vesshelm:
    helm_args: "upgrade --install {{ name }} {{ destination }} -n {{ namespace }}"
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
    setup_mock_helm_initial(&temp);

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

vesshelm:
    helm_args: "upgrade --install {{ name }} {{ destination }} -n {{ namespace }}"
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
    setup_mock_helm_initial(&temp);

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

vesshelm:
    helm_args: "upgrade --install {{ name }} {{ destination }} -n {{ namespace }}"
"#;
    fs::write(&config_path, config_content).unwrap();

    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("vesshelm"));
    cmd.current_dir(temp.path())
        .env("PATH", temp.path())
        .arg("deploy")
        .arg("--no-interactive")
        .arg("--no-progress")
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

vesshelm:
    helm_args: "upgrade --install {{ name }} {{ destination }} -n {{ namespace }}"

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

vesshelm:
    helm_args: "upgrade --install {{ name }} {{ destination }} -n {{ namespace }}"

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

vesshelm:
    helm_args: "upgrade --install {{ name }} {{ destination }} -n {{ namespace }}"

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

#[test]
fn test_deploy_helm_failure() {
    let temp_dir = tempfile::tempdir().unwrap();
    let bin_dir = temp_dir.path().join("bin");
    fs::create_dir(&bin_dir).unwrap();

    let helm_path = bin_dir.join("helm");
    // Mock helm failure
    let helm_script = r#"#!/bin/sh
echo "Mock helm failure"
exit 1
"#;
    fs::write(&helm_path, helm_script).unwrap();

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&helm_path).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&helm_path, perms).unwrap();
    }

    let config_path = temp_dir.path().join("vesshelm.yaml");
    let config_content = r#"
repositories:
  - name: stable
    url: https://charts.helm.sh/stable
charts:
  - name: my-chart
    repo_name: stable
    version: 1.0.0
    namespace: default
destinations:
  - name: default
    path: ./charts
vesshelm:
  helm_args: "upgrade"
"#;
    fs::write(&config_path, config_content).unwrap();

    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("vesshelm"));
    let path_env = std::env::var("PATH").unwrap_or_default();
    let new_path = format!("{}:{}", bin_dir.display(), path_env);

    cmd.current_dir(&temp_dir)
        .env("PATH", new_path)
        .arg("deploy")
        .arg("--no-interactive")
        .arg("--no-progress")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "Deployment failed for some charts",
        ));
}

#[test]
fn test_deploy_bad_dest_override() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config_path = temp_dir.path().join("vesshelm.yaml");
    let config_content = r#"
repositories:
  - name: stable
    url: https://charts.helm.sh/stable
charts:
  - name: my-chart
    repo_name: stable
    version: 1.0.0
    namespace: default
    destination_override: invalid-dest
destinations:
  - name: default
    path: ./charts
vesshelm:
  helm_args: "upgrade"
"#;
    fs::write(&config_path, config_content).unwrap();

    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("vesshelm"));
    cmd.current_dir(&temp_dir)
        .arg("deploy")
        .arg("--no-interactive")
        .arg("--no-progress")
        .assert()
        .failure()
        .failure()
        .stderr(predicate::str::contains(
            "Deployment failed for some charts",
        ));
}
fn setup_mock_helm_diff(temp_dir: &TempDir) {
    let mock_helm_path = temp_dir.path().join("helm");
    let mock_helm_content = r#"#!/bin/sh
if [ "$1" = "diff" ]; then
    echo "Mock Helm Diff: $@"
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
fn test_deploy_diff_enabled_false() {
    let temp = TempDir::new().unwrap();
    setup_mock_helm_diff(&temp);

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

vesshelm:
    helm_args: "upgrade"
    diff_enabled: false
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
        // Should NOT run diff
        .stdout(predicates::str::contains("Mock Helm Diff").not())
        .stdout(predicates::str::contains("Mock Helm called with: upgrade"));
}

#[test]
fn test_deploy_diff_args() {
    let temp = TempDir::new().unwrap();
    setup_mock_helm_diff(&temp);

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

vesshelm:
    helm_args: "upgrade"
    diff_enabled: true
    diff_args: "diff custom args"
"#;
    fs::write(&config_path, config_content).unwrap();

    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("vesshelm"));
    cmd.current_dir(temp.path())
        .env("PATH", temp.path())
        .arg("deploy")
        .arg("--no-interactive")
        .arg("--no-progress")
        .arg("--dry-run") // Ensure diff runs even if diff_enabled is default on, forcing validation
        .assert()
        .success()
        .stdout(predicates::str::contains(
            "Mock Helm Diff: diff custom args",
        ));
}

#[test]
fn test_deploy_values_files() {
    let temp = TempDir::new().unwrap();
    setup_mock_helm_diff(&temp);

    // Create values file
    let values_path = temp.path().join("values.yaml");
    fs::write(&values_path, "key: value").unwrap();

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
    values_files:
      - values.yaml

destinations:
  - name: default
    path: ./charts

vesshelm:
    helm_args: "upgrade"
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
        .stdout(predicates::str::contains("-f values.yaml"));
}

#[test]
fn test_deploy_inline_values() {
    let temp = TempDir::new().unwrap();
    setup_mock_helm_diff(&temp);

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
    values:
      - foo: bar

destinations:
  - name: default
    path: ./charts

vesshelm:
    helm_args: "upgrade"
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
        // verify -f and .yaml extension for temp file
        .stdout(predicates::str::contains(" -f ").and(predicates::str::contains(".yaml")));
}

#[test]
fn test_deploy_depends_order() {
    let temp = TempDir::new().unwrap();
    let mock_helm_path = temp.path().join("helm");
    // This script appends the chart name to a log file
    let mock_helm_content = r#"#!/bin/sh
if [ "$1" = "diff" ]; then exit 0; fi # skip diffs
# Last arg is usually dest/name or name, depend on template.
# But we can grep the full command.
echo "$@" >> deploy.log
"#;
    fs::write(&mock_helm_path, mock_helm_content).unwrap();
    let mut perms = fs::metadata(&mock_helm_path).unwrap().permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&mock_helm_path, perms).unwrap();

    let config_path = temp.path().join("vesshelm.yaml");
    let config_content = r#"
repositories:
  - name: stable
    url: https://charts.helm.sh/stable

charts:
  - name: dependent
    repo_name: stable
    version: 1.0.0
    namespace: default
    depends:
        - dependency

  - name: dependency
    repo_name: stable
    version: 1.0.0
    namespace: default

destinations:
  - name: default
    path: ./charts

vesshelm:
    helm_args: "upgrade {{ name }}"
"#;
    fs::write(&config_path, config_content).unwrap();

    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("vesshelm"));
    cmd.current_dir(temp.path())
        .env("PATH", temp.path())
        .arg("deploy")
        .arg("--no-interactive")
        .arg("--no-progress")
        .arg("--force") // Force deployment to ensure they run
        .assert()
        .success();

    let log_content = fs::read_to_string(temp.path().join("deploy.log")).unwrap();
    let lines: Vec<&str> = log_content.trim().split('\n').collect();

    // dependency should be before dependent
    let dep_idx = lines
        .iter()
        .position(|l| l.contains("dependency"))
        .expect("dependency not found");
    let dependent_idx = lines
        .iter()
        .position(|l| l.contains("dependent"))
        .expect("dependent not found");

    assert!(dep_idx < dependent_idx, "Dependency was not deployed first");
}
fn setup_mock_helm_simple(temp_dir: &TempDir) {
    let mock_helm_path = temp_dir.path().join("helm");
    let mock_helm_content = r#"#!/bin/sh
echo "Mock Helm called with: $@"
"#;
    fs::write(&mock_helm_path, mock_helm_content).unwrap();
    let mut perms = fs::metadata(&mock_helm_path).unwrap().permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&mock_helm_path, perms).unwrap();
}

#[test]
fn test_deploy_helm_args_override() {
    let temp = TempDir::new().unwrap();
    setup_mock_helm_simple(&temp);

    let config_path = temp.path().join("vesshelm.yaml");
    let config_content = r#"
repositories:
  - name: stable
    url: https://charts.helm.sh/stable

charts:
  - name: chart-override
    repo_name: stable
    version: 1.0.0
    namespace: default
    helm_args_override: "install custom-args"

destinations:
  - name: default
    path: ./charts

vesshelm:
    helm_args: "upgrade"
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
            "Mock Helm called with: install custom-args",
        ));
}

#[test]
fn test_deploy_helm_args_append() {
    let temp = TempDir::new().unwrap();
    setup_mock_helm_simple(&temp);

    let config_path = temp.path().join("vesshelm.yaml");
    let config_content = r#"
repositories:
  - name: stable
    url: https://charts.helm.sh/stable

charts:
  - name: chart-append
    repo_name: stable
    version: 1.0.0
    namespace: default
    helm_args_append: "--extra-flag"

destinations:
  - name: default
    path: ./charts

vesshelm:
    helm_args: "upgrade --install"
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
            "Mock Helm called with: upgrade --install --extra-flag",
        ));
}

#[test]
fn test_deploy_take_ownership_integration() {
    let temp = TempDir::new().unwrap();
    setup_mock_helm_simple(&temp);

    let config_path = temp.path().join("vesshelm.yaml");
    let config_content = r#"
repositories:
  - name: stable
    url: https://charts.helm.sh/stable
charts:
  - name: chart-ownership
    repo_name: stable
    version: 1.0.0
    namespace: default

destinations:
  - name: default
    path: ./charts

vesshelm:
    helm_args: "upgrade"
"#;
    fs::write(&config_path, config_content).unwrap();

    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("vesshelm"));
    cmd.current_dir(temp.path())
        .env("PATH", temp.path())
        .arg("deploy")
        .arg("--no-interactive")
        .arg("--no-progress")
        .arg("--take-ownership")
        .assert()
        .success()
        .stdout(predicates::str::contains(
            "Mock Helm called with: upgrade --take-ownership",
        ));
}

fn setup_mock_helm_interpolation(temp_dir: &TempDir) {
    let mock_helm_path = temp_dir.path().join("helm");
    let mock_helm_content = r#"#!/bin/sh
# Echo all args
echo "Mock Helm called with: $@"

# Loop to find -f and cat them
prev=""
for arg in "$@"; do
    if [ "$prev" = "-f" ]; then
        echo "--> CONTENT OF $arg:"
        cat "$arg"
    fi
    prev="$arg"
done

if [ "$1" = "diff" ]; then
    exit 0
fi
"#;
    fs::write(&mock_helm_path, mock_helm_content).unwrap();
    let mut perms = fs::metadata(&mock_helm_path).unwrap().permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&mock_helm_path, perms).unwrap();
}

#[test]
fn test_deploy_local_chart_interpolation() {
    let temp = TempDir::new().unwrap();
    setup_mock_helm_interpolation(&temp);

    // Create variables file
    let vars_path = temp.path().join("vars.yaml");
    fs::write(&vars_path, "global_var: injected_value").unwrap();

    // Create local chart structure
    let chart_dir = temp.path().join("charts").join("local-chart");
    fs::create_dir_all(&chart_dir).unwrap();
    fs::write(
        chart_dir.join("Chart.yaml"),
        "name: local-chart\nversion: 0.1.0",
    )
    .unwrap();

    // value with variable
    fs::write(chart_dir.join("values.yaml"), "key: {{ global_var }}").unwrap();

    let config_path = temp.path().join("vesshelm.yaml");
    let config_content = r#"
repositories: []
variable_files:
  - vars.yaml
charts:
  - name: local-chart
    chart_path: charts/local-chart
    namespace: default
destinations:
  - name: default
    path: ./charts
vesshelm:
    helm_args: "upgrade"
"#;
    fs::write(&config_path, config_content).unwrap();

    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("vesshelm"));
    let path_env = std::env::var("PATH").unwrap_or_default();
    let new_path = format!("{}:{}", temp.path().display(), path_env);

    cmd.current_dir(temp.path())
        .env("PATH", new_path)
        .arg("deploy")
        .arg("--no-interactive")
        .arg("--no-progress")
        .arg("--dry-run")
        .assert()
        .success()
        // Verify we see the injected value echoed by mock helm
        .stdout(predicates::str::contains("--> CONTENT OF"))
        .stdout(predicates::str::contains("key: injected_value"));
}

#[test]
fn test_deploy_missing_variable_error() {
    let temp = TempDir::new().unwrap();
    setup_mock_helm_simple(&temp);

    // Create variables file (empty or unrelated)
    let vars_path = temp.path().join("vars.yaml");
    fs::write(&vars_path, "other: val").unwrap();

    // Create local chart structure
    let chart_dir = temp.path().join("charts").join("bad-chart");
    fs::create_dir_all(&chart_dir).unwrap();
    fs::write(
        chart_dir.join("Chart.yaml"),
        "name: bad-chart\nversion: 0.1.0",
    )
    .unwrap();

    // value with missing variable
    fs::write(chart_dir.join("values.yaml"), "key: {{ missing_var }}").unwrap();

    let config_path = temp.path().join("vesshelm.yaml");
    let config_content = r#"
repositories: []
variable_files:
  - vars.yaml
charts:
  - name: bad-chart
    chart_path: charts/bad-chart
    namespace: default
destinations:
  - name: default
    path: ./charts
vesshelm:
    helm_args: "upgrade"
"#;
    fs::write(&config_path, config_content).unwrap();

    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("vesshelm"));
    cmd.current_dir(temp.path())
        .env("PATH", temp.path())
        .arg("deploy")
        .arg("--no-interactive")
        .arg("--no-progress")
        .assert()
        .failure()
        .stdout(predicates::str::contains(
            "Failed to render local values.yaml",
        ));
}
