use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;

#[test]
fn test_sync_helm_repo_failure() {
    let temp_dir = tempfile::tempdir().unwrap();
    let bin_dir = temp_dir.path().join("bin");
    fs::create_dir(&bin_dir).unwrap();

    let helm_path = bin_dir.join("helm");
    // Mock helm failure for repo update
    let helm_script = r#"#!/bin/sh
if [ "$1" = "repo" ] && [ "$2" = "update" ]; then
    echo "Mock helm repo update failed"
    exit 1
fi
echo "Unknown helm command $*"
exit 0
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
"#;
    fs::write(&config_path, config_content).unwrap();

    // Create chart dir - actually sync downloads it.
    // Sync needs to run.

    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("vesshelm"));
    let path_env = std::env::var("PATH").unwrap_or_default();
    let new_path = format!("{}:{}", bin_dir.display(), path_env);

    cmd.current_dir(&temp_dir)
        .env("PATH", new_path)
        .arg("sync")
        .arg("--no-progress")
        .assert()
        .failure() // Should fail if repo update fails?
        // Sync engine currently logs warning for repo update failure but continues?
        // Let's check sync.rs line 57 code.
        // It prints warning.
        // Then continues to sync charts.
        // If charts rely on repo, they fail later.
        // If mock helm is just failing repo update, chart sync (helm dependency build) might pass (mock returns access/exit 0 for other).
        // But `test_sync_helm_repo_failure` implies checking if warning is printed.
        // And if all charts succeed, exit code is 0.
        .failure()
        .stdout(predicate::str::contains("Failed to update helm repos"));
}

#[test]
fn test_sync_chart_failure() {
    let temp_dir = tempfile::tempdir().unwrap();
    let bin_dir = temp_dir.path().join("bin");
    fs::create_dir(&bin_dir).unwrap();

    let helm_path = bin_dir.join("helm");
    // Mock helm failure for dependency build (which calls `helm dependency update` or `helm dependency build`)
    // Client calls `helm dependency build`.
    let helm_script = r#"#!/bin/sh
if [ "$1" = "repo" ]; then exit 0; fi
if [ "$1" = "dependency" ]; then
    echo "Mock helm dependency failed"
    exit 1
fi
exit 0
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
"#;
    fs::write(&config_path, config_content).unwrap();

    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("vesshelm"));
    let path_env = std::env::var("PATH").unwrap_or_default();
    let new_path = format!("{}:{}", bin_dir.display(), path_env);

    cmd.current_dir(&temp_dir)
        .env("PATH", new_path)
        .arg("sync")
        .arg("--no-progress")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Some charts failed to sync"))
        .stdout(predicate::str::contains("[FAIL]"));
}
