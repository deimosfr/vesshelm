use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;

#[test]
fn test_uninstall_success() {
    let temp_dir = tempfile::tempdir().unwrap();
    let bin_dir = temp_dir.path().join("bin");
    fs::create_dir(&bin_dir).unwrap();

    // Mock helm script
    let helm_path = bin_dir.join("helm");
    let helm_script = r#"#!/bin/sh
if [ "$1" = "uninstall" ]; then
    echo "Mock helm uninstall $2 -n $4"
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

    // Create vesshelm.yaml
    let config_path = temp_dir.path().join("vesshelm.yaml");
    let config_content = r#"
repositories:
  - name: stable
    url: https://example.com/stable
charts:
  - name: my-chart
    repo_name: stable
    version: 1.0.0
    namespace: my-ns
    dest: default
destinations:
  - name: default
    path: ./charts
"#;
    fs::write(&config_path, config_content).unwrap();

    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("vesshelm"));

    // Add bin_dir to PATH
    let path_env = std::env::var("PATH").unwrap_or_default();
    let new_path = format!("{}:{}", bin_dir.display(), path_env);

    // Run uninstall with --yes to skip interactive confirmation
    // Wait, uninstall command uses `dialoguer::Confirm` which interacts with stdin.
    // I can pipe "y\n" to stdin.
    cmd.current_dir(&temp_dir)
        .env("PATH", new_path)
        .arg("uninstall")
        .arg("my-chart")
        .arg("--no-interactive")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Successfully uninstalled my-chart",
        ));
}

#[test]
fn test_uninstall_not_found() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config_path = temp_dir.path().join("vesshelm.yaml");
    let config_content = r#"
repositories: []
charts: []
destinations: []
"#;
    fs::write(&config_path, config_content).unwrap();

    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("vesshelm"));

    cmd.current_dir(&temp_dir)
        .arg("uninstall")
        .arg("missing-chart")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Chart 'missing-chart' not found"));
}
