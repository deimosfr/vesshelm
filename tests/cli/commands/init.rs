use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;

#[test]
fn test_init_creates_config() {
    let temp_dir = tempfile::tempdir().unwrap();
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("vesshelm"));

    cmd.current_dir(&temp_dir)
        .arg("init")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Created default configuration file",
        ));

    let config_path = temp_dir.path().join("vesshelm.yaml");
    assert!(config_path.exists());

    let content = fs::read_to_string(config_path).unwrap();
    assert!(content.contains("repositories:"));
    assert!(content.contains("repositories:"));
    assert!(content.contains("destinations:"));
    assert!(content.contains("helm:"));
    assert!(content.contains("args:"));
    assert!(content.contains("upgrade --install"));
    assert!(content.contains("--create-namespace"));
    assert!(content.contains("diff_enabled: true"));
}

#[test]
fn test_init_existing_config() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config_path = temp_dir.path().join("vesshelm.yaml");
    fs::write(&config_path, "existing content").unwrap();

    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("vesshelm"));

    cmd.current_dir(&temp_dir)
        .arg("init")
        .assert()
        .success()
        .stdout(predicate::str::contains("already exists"));

    let content = fs::read_to_string(config_path).unwrap();
    assert_eq!(content, "existing content");
}
