use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

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
        .arg("chart-a")
        .assert()
        .success()
        // Should say "Syncing 1 charts..." not 2
        .stdout(predicate::str::contains("Syncing 1 charts"));
}

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

#[test]
fn test_sync_force() {
    let temp_dir = TempDir::new().unwrap();
    let bin_dir = temp_dir.path().join("bin");
    fs::create_dir(&bin_dir).unwrap();

    // Mock helm
    let helm_path = bin_dir.join("helm");
    let helm_script = r#"#!/bin/sh
if [ "$1" = "pull" ]; then
    # Create dummy chart files
    for last_arg in "$@"; do :; done
    mkdir -p "$last_arg/chart-a"
    echo "name: chart-a" > "$last_arg/chart-a/Chart.yaml"
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

destinations:
  - name: default
    path: ./charts
"#;
    fs::write(&config_path, config_content).unwrap();

    // First sync to establish "up to date" state
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("vesshelm"));
    let path_env = std::env::var("PATH").unwrap_or_default();
    let new_path = format!("{}:{}", bin_dir.display(), path_env);

    cmd.current_dir(&temp_dir)
        .env("PATH", &new_path)
        .arg("sync")
        .assert()
        .success();

    // Second sync should skip
    let output2 = std::process::Command::new(assert_cmd::cargo::cargo_bin!("vesshelm"))
        .current_dir(&temp_dir)
        .env("PATH", &new_path)
        .arg("sync")
        .output()
        .expect("Failed to execute cmd2");

    assert!(output2.status.success(), "cmd2 failed");
    let stdout2 = String::from_utf8_lossy(&output2.stdout);
    assert!(
        stdout2.contains("Skipped: 1"),
        "cmd2 did not skip. Output:\n{}",
        stdout2
    );

    // Third sync with --force should NOT skip
    // We use std::process::Command to ensure arguments are passed correctly, bypassing potential assert_cmd issues
    let bin_path = assert_cmd::cargo::cargo_bin!("vesshelm");

    let output = std::process::Command::new(bin_path)
        .current_dir(&temp_dir)
        .env("PATH", &new_path)
        .arg("sync")
        .arg("--ignore-skip")
        .output()
        .expect("Failed to execute command");

    assert!(
        output.status.success(),
        "Command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Check summary statistics instead of fragile progress output
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("Synced:  1"),
        "Output did not indicate 1 chart synced. Full output:\n{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        !String::from_utf8_lossy(&output.stdout).contains("Skipped: 1"),
        "Output indicated skip (but should force sync). Full output:\n{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn test_sync_no_sync() {
    let temp_dir = TempDir::new().unwrap();
    let bin_dir = temp_dir.path().join("bin");
    fs::create_dir(&bin_dir).unwrap();

    // Mock helm
    let helm_path = bin_dir.join("helm");
    let helm_script = r#"#!/bin/sh
if [ "$1" = "pull" ]; then
    for last_arg in "$@"; do :; done
    mkdir -p "$last_arg/chart-nosync"
    echo "name: chart-nosync" > "$last_arg/chart-nosync/Chart.yaml"
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
  - name: chart-nosync
    repo_name: stable
    version: 1.0.0
    namespace: default
    dest: default
    no_sync: true

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
        .arg("sync")
        .arg("--no-progress")
        .assert()
        .success()
        // Should show skipped reason no_sync=true
        .stdout(predicates::str::contains("no_sync=true"))
        .stdout(predicates::str::contains("Skipped: 1"));
}

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
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("vesshelm"));
    cmd.current_dir(temp_dir.path())
        .env("PATH", &new_path)
        .arg("--no-progress")
        .arg("sync")
        .assert()
        .success();

    let lockfile_path = temp_dir.path().join("vesshelm.lock");
    assert!(lockfile_path.exists(), "Lockfile should exist");

    // 2nd Run: Should skip
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("vesshelm"));
    cmd.current_dir(temp_dir.path())
        .env("PATH", &new_path)
        .arg("--no-progress")
        .arg("sync")
        .assert()
        .success()
        .stdout(predicate::str::contains("up to date"));

    // 3rd Run with flag: Should NOT skip
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("vesshelm"));
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

    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("vesshelm"));
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

#[test]
fn test_sync_oci_repo() {
    let temp_dir = TempDir::new().unwrap();
    let bin_dir = temp_dir.path().join("bin");
    fs::create_dir(&bin_dir).unwrap();

    // Mock helm
    let helm_path = bin_dir.join("helm");
    let helm_script = r#"#!/bin/sh
if [ "$1" = "pull" ]; then
    # Expect oci:// URL
    if echo "$2" | grep -q "^oci://"; then
        # Create dummy chart files
        for last_arg in "$@"; do :; done
        mkdir -p "$last_arg/chart-oci"
        echo "name: chart-oci" > "$last_arg/chart-oci/Chart.yaml"
        exit 0
    else
        echo "Error: Expected OCI URL, got $2"
        exit 1
    fi
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
  - name: my-oci
    url: oci://registry.example.com/charts
    type: oci

charts:
  - name: chart-oci
    repo_name: my-oci
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
        .arg("sync")
        .assert()
        .success()
        .stdout(predicates::str::contains("Synced:  1"));
}
