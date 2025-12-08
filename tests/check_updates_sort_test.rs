use anyhow::Result;
use std::env;
use std::fs;
use std::process::Command;
use tempfile::TempDir;

#[test]
fn test_check_updates_sorted_output() -> Result<()> {
    // Create temp dir for test config
    let temp_dir = TempDir::new()?;
    let config_path = temp_dir.path().join("vesshelm.yaml");

    // Create config with unsorted charts
    let config_content = r#"
repositories:
  - name: stable
    url: https://charts.helm.sh/stable
charts:
  - name: zebra
    repo_name: stable
    version: 1.0.0
    namespace: default
  - name: alpha
    repo_name: stable
    version: 1.0.0
    namespace: default
  - name: beta
    repo_name: stable
    version: 1.0.0
    namespace: default
destinations:
  - name: default
    path: ./charts
"#;
    fs::write(&config_path, config_content)?;

    // Run check-updates command via cargo run to capture stdout
    // We use proper cargo command to run the binary against the test config
    let output = Command::new(env!("CARGO_BIN_EXE_vesshelm"))
        .arg("update")
        .arg("--jobs")
        .arg("1")
        .arg("--config")
        .arg(config_path)
        .env("PATH", env::var("PATH").unwrap_or_default()) // Ensure helm matches (mocks might be needed if integration test)
        // Wait, running the real binary requires real helm and network unless we mock.
        // Integration tests usually invoke the binary.
        // However, this might fail if no internet or 'helm' not in path.
        // But for sorting check, we might just need to see the "checking [name]..." output order.
        // Even if network fails, the "checking..." line prints BEFORE the check.
        .output()?;

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Extract lines starting with "checking"
    let check_lines: Vec<&str> = stdout
        .lines()
        .filter(|line| line.contains("checking"))
        .collect();

    // Verify order: alpha, beta, zebra
    // Note: The output might contain ANSI codes or extra text, but the order of appearance matters.

    // We expect 3 checks
    assert!(check_lines.len() >= 3, "Output was:\n{}", stdout);

    let alpha_pos = stdout.find("checking alpha...");
    let beta_pos = stdout.find("checking beta...");
    let zebra_pos = stdout.find("checking zebra...");

    assert!(alpha_pos.is_some());
    assert!(beta_pos.is_some());
    assert!(zebra_pos.is_some());

    let a = alpha_pos.unwrap();
    let b = beta_pos.unwrap();
    let z = zebra_pos.unwrap();

    assert!(a < b, "alpha should come before beta");
    assert!(b < z, "beta should come before zebra");

    Ok(())
}
