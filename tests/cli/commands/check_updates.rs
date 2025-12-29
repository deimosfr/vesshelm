use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_check_updates_helm_outdated() {
    let temp_dir = tempfile::tempdir().unwrap();
    let bin_dir = temp_dir.path().join("bin");
    fs::create_dir(&bin_dir).unwrap();

    let helm_path = bin_dir.join("helm");
    // Indentation matters for YAML
    let helm_search_output = "
- name: stable/nginx
  version: 1.2.0
  app_version: 1.16.0
  description: Nginx chart";

    // We construct the script carefully to preserve newlines in the mocked output
    let helm_script = format!(
        r#"#!/bin/sh
if [ "$1" = "repo" ] && [ "$2" = "update" ]; then
    echo "Mock helm repo update"
    exit 0
fi
if [ "$1" = "search" ] && [ "$2" = "repo" ]; then
    cat <<EOF
{}
EOF
    exit 0
fi
echo "Unknown helm command $*"
exit 1
"#,
        helm_search_output
    );

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
  - name: nginx
    repo_name: stable
    version: 1.0.0
    namespace: default
    dest: default
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
        .arg("check-updates")
        .assert()
        .success()
        .stdout(predicate::str::contains("Outdated"))
        .stdout(predicate::str::contains("1.0.0 -> 1.2.0"));
}

#[test]
fn test_check_updates_apply() {
    let temp_dir = tempfile::tempdir().unwrap();
    let bin_dir = temp_dir.path().join("bin");
    fs::create_dir(&bin_dir).unwrap();

    let helm_path = bin_dir.join("helm");
    let helm_search_output = "
- name: stable/nginx
  version: 1.2.0
";
    let helm_script = format!(
        r#"#!/bin/sh
if [ "$1" = "repo" ] && [ "$2" = "update" ]; then exit 0; fi
if [ "$1" = "search" ]; then
    cat <<EOF
{}
EOF
    exit 0;
fi
exit 0
"#,
        helm_search_output
    );
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
  - name: nginx
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
        .arg("check-updates")
        .arg("--apply")
        .assert()
        .success()
        .stdout(predicate::str::contains("vesshelm.yaml updated"));

    let new_content = fs::read_to_string(&config_path).unwrap();
    assert!(new_content.contains("version: 1.2.0"));
}

#[test]
fn test_check_updates_helm_failure() {
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
"#;
    fs::write(&config_path, config_content).unwrap();

    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("vesshelm"));
    let path_env = std::env::var("PATH").unwrap_or_default();
    let new_path = format!("{}:{}", bin_dir.display(), path_env);

    cmd.current_dir(&temp_dir)
        .env("PATH", new_path)
        .arg("check-updates")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "Failed to update helm repositories",
        ));
}

#[test]
fn test_check_updates_only_flag() {
    let temp_dir = TempDir::new().unwrap();
    let bin_dir = temp_dir.path().join("bin");
    fs::create_dir(&bin_dir).unwrap();

    let helm_path = bin_dir.join("helm");
    // Mock diff output (not relevant for this test but good practice) and search output
    let helm_script = r#"#!/bin/sh
if [ "$1" = "search" ]; then
   # We don't care about versions here, just that it runs.
   # But to simulate output we need to be careful.
   # Only return info if chart exists in the "repo".
   if echo "$*" | grep -q "chart-a"; then
       echo "- name: stable/chart-a\n  version: 2.0.0"
   elif echo "$*" | grep -q "chart-b"; then
       echo "- name: stable/chart-b\n  version: 2.0.0"
   fi
   exit 0
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
  - name: chart-a
    repo_name: stable
    version: 1.0.0
    namespace: default
    dest: default
  - name: chart-b
    repo_name: stable
    version: 1.0.0
    namespace: default
    dest: default

destinations:
  - name: default
    path: ./charts
"#;
    fs::write(&config_path, config_content).unwrap();

    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("vesshelm"));
    let path_env = std::env::var("PATH").unwrap_or_default();
    let new_path = format!("{}:{}", bin_dir.display(), path_env);

    cmd.current_dir(&temp_dir)
        .env("PATH", &new_path)
        .arg("check-updates")
        .arg("chart-a")
        .assert()
        .success()
        .stdout(predicate::str::contains("checking chart-a..."))
        .stdout(predicate::str::contains("checking chart-b...").not());
}
