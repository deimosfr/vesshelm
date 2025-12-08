use anyhow::Result;
use std::fs::File;
use std::io::Write;
use tempfile::tempdir;
use vesshelm::config::Config;

#[test]
fn test_config_secrets_files_parsing() -> Result<()> {
    let dir = tempdir()?;
    let config_path = dir.path().join("vesshelm.yaml");
    let secret_file_path = dir.path().join("secrets.yaml");

    // Create a dummy secret file so validation passes
    File::create(&secret_file_path)?.write_all(b"secret: value")?;

    let config_content = format!(
        r#"
repositories: []
charts: []
destinations: []
secrets_files:
  - "{}"
"#,
        secret_file_path.to_str().unwrap()
    );

    File::create(&config_path)?.write_all(config_content.as_bytes())?;

    let config = Config::load_from_path(&config_path)?;

    assert!(config.secrets_files.is_some());
    let secrets = config.secrets_files.unwrap();
    assert_eq!(secrets.len(), 1);
    assert_eq!(secrets[0], secret_file_path.to_str().unwrap());

    Ok(())
}

#[test]
fn test_config_secrets_files_validation_fails_if_missing() -> Result<()> {
    let dir = tempdir()?;
    let config_path = dir.path().join("vesshelm.yaml");
    let missing_secret_path = dir.path().join("missing_secrets.yaml");

    let config_content = format!(
        r#"
repositories: []
charts: []
destinations: []
secrets_files:
  - "{}"
"#,
        missing_secret_path.to_str().unwrap()
    );

    File::create(&config_path)?.write_all(config_content.as_bytes())?;

    let result = Config::load_from_path(&config_path);

    assert!(result.is_err());
    let err = result.unwrap_err();
    let err_str = format!("{:?}", err);
    println!("Error details: {}", err_str);
    assert!(err_str.contains("secrets_file_not_found"));

    Ok(())
}
