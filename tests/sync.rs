use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;

#[test]
fn test_sync_helm() {
    let temp_dir = tempfile::tempdir().unwrap();
    let bin_dir = temp_dir.path().join("bin");
    fs::create_dir(&bin_dir).unwrap();

    // Mock helm script
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
    mkdir -p "$last_arg/nginx"
    touch "$last_arg/nginx/Chart.yaml"
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

    // Create vesshelm.yaml
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

    // Add bin_dir to PATH
    let path_env = std::env::var("PATH").unwrap_or_default();
    let new_path = format!("{}:{}", bin_dir.display(), path_env);

    cmd.current_dir(&temp_dir)
        .env("PATH", new_path)
        .arg("sync")
        .assert()
        .success()
        .stdout(predicate::str::contains("Summary:"));

    // Verify chart was "downloaded" to destination
    let chart_yaml = temp_dir.path().join("charts/nginx/Chart.yaml");
    assert!(
        chart_yaml.exists(),
        "Chart.yaml should exist at {:?}",
        chart_yaml
    );
}

#[test]
fn test_sync_git() {
    let temp_dir = tempfile::tempdir().unwrap();
    let upstream_dir = temp_dir.path().join("upstream");
    let repo_dir = upstream_dir.join("my-chart-repo");
    fs::create_dir_all(&repo_dir).unwrap();

    // Initialize upstream git repo
    let repo = git2::Repository::init(&repo_dir).unwrap();
    let mut index = repo.index().unwrap();

    // Add a chart file
    let chart_path = repo_dir.join("my-chart/Chart.yaml");
    fs::create_dir_all(chart_path.parent().unwrap()).unwrap();
    fs::write(&chart_path, "name: my-chart\nversion: 0.1.0").unwrap();

    index
        .add_path(std::path::Path::new("my-chart/Chart.yaml"))
        .unwrap();
    let oid = index.write_tree().unwrap();
    let tree = repo.find_tree(oid).unwrap();
    let sig = git2::Signature::now("Test User", "test@example.com").unwrap();
    repo.commit(Some("HEAD"), &sig, &sig, "Initial commit", &tree, &[])
        .unwrap();

    // Create vesshelm.yaml
    let config_path = temp_dir.path().join("vesshelm.yaml");
    let config_content = format!(
        r#"
repositories:
  - name: my-git-repo
    url: file://{}
    type: git

charts:
  - name: my-chart-dest
    repo_name: my-git-repo
    version: HEAD
    namespace: default
    dest: default
    chart_path: my-chart

destinations:
  - name: default
    path: ./charts
"#,
        repo_dir.display()
    );
    fs::write(&config_path, config_content).unwrap();

    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("vesshelm"));

    cmd.current_dir(&temp_dir)
        .arg("sync")
        .assert()
        .success()
        .stdout(predicate::str::contains("Summary:"));

    // Verify chart was "downloaded" to destination
    let chart_yaml = temp_dir.path().join("charts/my-chart-dest/Chart.yaml");
    assert!(
        chart_yaml.exists(),
        "Chart.yaml should exist at {:?}",
        chart_yaml
    );
}

#[test]
fn test_sync_invalid_config() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config_path = temp_dir.path().join("vesshelm.yaml");
    // Invalid because chart refers to non-existent repo "missing-repo"
    fs::write(
        &config_path,
        r#"
repositories:
  - name: my-repo
    url: https://charts.bitnami.com/bitnami
charts:
  - name: nginx
    repo_name: missing-repo
    version: 13.2.1
    namespace: default
    dest: default
destinations:
  - name: default
    path: ./charts
"#,
    )
    .unwrap();

    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("vesshelm"));

    cmd.current_dir(&temp_dir)
        .arg("sync")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Configuration validation failed"));
}

#[test]
fn test_sync_filtered_count() {
    let temp_dir = tempfile::tempdir().unwrap();
    let bin_dir = temp_dir.path().join("bin");
    fs::create_dir(&bin_dir).unwrap();

    // Mock helm script
    let helm_path = bin_dir.join("helm");
    // Minimal mock for sync that creates the chart directory
    let helm_script = r#"#!/bin/sh
if [ "$1" = "pull" ]; then
    # helm pull repo/chart ... --untardir dir
    # Last arg is likely untardir if we follow sync.rs logic, but sync.rs does:
    # arg("--untardir").arg(temp_path)
    # So untardir is the last argument.
    
    # We need to find the untardir.
    # A simple way for this specific test where we know the args:
    # args are: pull stable/chart-a --version 1.0.0 --untar --untardir <temp_path>
    
    # Get the last argument
    for last_arg in "$@"; do :; done
    
    # Check which chart is being pulled to create right dir
    if echo "$@" | grep -q "chart-a"; then
        mkdir -p "$last_arg/chart-a"
    fi
     if echo "$@" | grep -q "chart-b"; then
        mkdir -p "$last_arg/chart-b"
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

    // Create vesshelm.yaml with two charts
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

    // Add bin_dir to PATH
    let path_env = std::env::var("PATH").unwrap_or_default();
    let new_path = format!("{}:{}", bin_dir.display(), path_env);

    cmd.current_dir(&temp_dir)
        .env("PATH", new_path)
        .arg("sync")
        .arg("--only")
        .arg("chart-a")
        .assert()
        .success()
        // Should say "Syncing 1 charts..." not 2
        .stdout(predicate::str::contains("Syncing 1 charts"));
}
