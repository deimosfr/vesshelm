use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;

#[test]
fn test_validate_valid_config() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config_path = temp_dir.path().join("vesshelm.yaml");
    fs::write(
        &config_path,
        r#"
repositories:
  - name: my-repo
    url: https://charts.bitnami.com/bitnami
charts:
  - name: nginx
    repo_name: my-repo
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
        .arg("validate")
        .assert()
        .success()
        .stdout(predicate::str::contains("Configuration is valid"));
}

#[test]
fn test_validate_invalid_config() {
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
        .arg("validate")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "Chart references a repository that does not exist",
        ));
}

#[test]
fn test_validate_duplicate_chart_name() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config_path = temp_dir.path().join("vesshelm.yaml");
    // Invalid because of duplicate chart name "nginx" in same namespace "default"
    fs::write(
        &config_path,
        r#"
repositories:
  - name: my-repo
    url: https://charts.bitnami.com/bitnami
charts:
  - name: nginx
    repo_name: my-repo
    version: 13.2.1
    namespace: default
    dest: default
  - name: nginx
    repo_name: my-repo
    version: 13.2.2
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
        .arg("validate")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Duplicate chart detected"));
}

#[test]
fn test_validate_same_name_different_namespace_valid() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config_path = temp_dir.path().join("vesshelm.yaml");
    // Valid because chart names are same but namespaces are different
    fs::write(
        &config_path,
        r#"
repositories:
  - name: my-repo
    url: https://charts.bitnami.com/bitnami
charts:
  - name: nginx
    repo_name: my-repo
    version: 13.2.1
    namespace: default
    dest: default
  - name: nginx
    repo_name: my-repo
    version: 13.2.2
    namespace: other-ns
    dest: default
destinations:
  - name: default
    path: ./charts
"#,
    )
    .unwrap();

    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("vesshelm"));
    cmd.current_dir(&temp_dir)
        .arg("validate")
        .assert()
        .success();
}
