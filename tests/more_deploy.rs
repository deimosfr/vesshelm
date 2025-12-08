use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;

#[test]
fn test_deploy_missing_chart() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config_path = temp_dir.path().join("vesshelm.yaml");
    // Config with missing chart in whitelist
    let config_content = r#"
repositories: []
charts: []
destinations: []
helm:
  args: "upgrade"
"#;
    fs::write(&config_path, config_content).unwrap();

    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("vesshelm"));
    cmd.current_dir(&temp_dir)
        .arg("deploy")
        .arg("--only")
        .arg("missing-chart")
        .assert()
        .failure()
        .stderr(predicate::str::contains("missing-chart"));
}

#[test]
fn test_deploy_dry_run_success() {
    let temp_dir = tempfile::tempdir().unwrap();
    let bin_dir = temp_dir.path().join("bin");
    fs::create_dir(&bin_dir).unwrap();

    let helm_path = bin_dir.join("helm");
    // Mock helm diff
    let helm_script = r#"#!/bin/sh
if [ "$1" = "diff" ]; then
    echo "Mock helm diff output"
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
    dest: default
destinations:
  - name: default
    path: ./charts
helm:
  args: "upgrade --install"
"#;
    fs::write(&config_path, config_content).unwrap();

    // Create chart dir
    let chart_dir = temp_dir.path().join("charts/my-chart");
    fs::create_dir_all(&chart_dir).unwrap();

    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("vesshelm"));
    let path_env = std::env::var("PATH").unwrap_or_default();
    let new_path = format!("{}:{}", bin_dir.display(), path_env);

    cmd.current_dir(&temp_dir)
        .env("PATH", new_path)
        .arg("deploy")
        .arg("--dry-run")
        .arg("--no-progress")
        .assert()
        .success()
        .stdout(predicate::str::contains("Mock helm diff output"))
        .stdout(predicate::str::contains("Deployed: 0"))
        .stdout(predicate::str::contains("Ignored:  1"));
}
