use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;

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
helm:
  args: "upgrade"
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
helm:
  args: "upgrade"
"#;
    fs::write(&config_path, config_content).unwrap();

    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("vesshelm"));
    cmd.current_dir(&temp_dir)
        .arg("deploy")
        .arg("--no-interactive")
        .arg("--no-progress")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Configuration validation failed"))
        .stderr(predicate::str::contains(
            "Chart references a destination that does not exist",
        ));
}
