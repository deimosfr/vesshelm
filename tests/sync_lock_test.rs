use assert_cmd::Command as AssertCommand;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_sync_creates_lockfile_and_skips() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("vesshelm.yaml");

    // Create a dummy config
    let config = r#"
charts:
  - name: test-chart
    repo_name: stable
    version: 1.0.0
    namespace: default
    chart_path: charts/test-chart

repositories:
  - name: stable
    url: https://charts.helm.sh/stable
    type: helm

destinations:
  - name: default
    path: ./charts
"#;
    fs::write(&config_path, config).unwrap();

    // Mock helm script
    let bin_dir = temp_dir.path().join("bin");
    fs::create_dir(&bin_dir).unwrap();
    let helm_path = bin_dir.join("helm");
    let helm_script = r#"#!/bin/sh
if [ "$1" = "repo" ] && [ "$2" = "add" ]; then
    echo "Mock helm repo add $*"
    exit 0
fi
if [ "$1" = "pull" ]; then
    # Expected format: helm pull repo/chart --version v --untar --untardir dir
    # get the last argument (the directory)
    for last_arg in "$@"; do :; done
    mkdir -p "$last_arg/test-chart"
    touch "$last_arg/test-chart/Chart.yaml"
    echo "Mock helm pull $*"
    exit 0
fi
echo "Unknown helm command $*"
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

    // Add bin_dir to PATH
    let path_env = std::env::var("PATH").unwrap_or_default();
    let new_path = format!("{}:{}", bin_dir.display(), path_env);

    // 1st Run: Should create lockfile
    let mut cmd = AssertCommand::new(assert_cmd::cargo::cargo_bin!("vesshelm"));
    cmd.current_dir(temp_dir.path())
        .env("PATH", &new_path)
        .arg("--no-progress")
        .arg("sync")
        .assert()
        .success();

    let lockfile_path = temp_dir.path().join("vesshelm.lock");
    assert!(lockfile_path.exists(), "Lockfile should exist");

    // 2nd Run: Should skip
    let mut cmd = AssertCommand::new(assert_cmd::cargo::cargo_bin!("vesshelm"));
    cmd.current_dir(temp_dir.path())
        .env("PATH", &new_path)
        .arg("--no-progress")
        .arg("sync")
        .assert()
        .success()
        .stdout(predicate::str::contains("up to date"));

    // 3rd Run with flag: Should NOT skip
    let mut cmd = AssertCommand::new(assert_cmd::cargo::cargo_bin!("vesshelm"));
    cmd.current_dir(temp_dir.path())
        .env("PATH", &new_path)
        .arg("--no-progress")
        .arg("sync")
        .arg("--ignore-skip")
        .assert()
        .success()
        .stdout(predicate::str::contains("up to date").not());

    // 4th Run: Remove folder, should re-sync even if locked
    let chart_dir = temp_dir.path().join("charts/test-chart");
    fs::remove_dir_all(&chart_dir).unwrap();
    assert!(!chart_dir.exists());

    let mut cmd = AssertCommand::new(assert_cmd::cargo::cargo_bin!("vesshelm"));
    cmd.current_dir(temp_dir.path())
        .env("PATH", &new_path)
        .arg("--no-progress")
        .arg("sync")
        .assert()
        .success()
        // Should NOT say "up to date" because folder was missing
        .stdout(predicate::str::contains("up to date").not());

    assert!(chart_dir.exists(), "Chart directory should be restored");
}
