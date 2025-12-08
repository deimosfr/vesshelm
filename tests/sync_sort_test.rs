use anyhow::Result;
use std::env;
use std::fs;
use std::sync::Mutex;
use tempfile::TempDir;

// Mutex to prevent race conditions when modifying PATH
static PATH_LOCK: Mutex<()> = Mutex::new(());

fn setup_mock_helm(script: &str) -> (TempDir, String) {
    let temp_dir = tempfile::tempdir().unwrap();
    let bin_dir = temp_dir.path().join("bin");
    fs::create_dir(&bin_dir).unwrap();

    let helm_path = bin_dir.join("helm");
    fs::write(&helm_path, script).unwrap();

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&helm_path).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&helm_path, perms).unwrap();
    }

    let path_env = env::var("PATH").unwrap_or_default();
    let new_path = format!("{}:{}", bin_dir.display(), path_env);

    (temp_dir, new_path)
}

#[test]
fn test_sync_execution_order() -> Result<()> {
    let _lock = PATH_LOCK.lock().unwrap();

    // Mock script that appends the pulled chart name to a log file
    // We expect `helm pull <repo>/<chart> ...`
    // So we capture $2 (chart ref)
    // Actually args are: pull <chart_ref> --version ...
    // So $2 is chart_ref.
    let log_file_path = env::temp_dir().join("vesshelm_sync_order.log");
    let _ = fs::remove_file(&log_file_path);

    let script = format!(
        r#"#!/bin/sh
if [ "$1" = "repo" ]; then exit 0; fi
if [ "$1" = "pull" ]; then
    echo "$2" >> "{}"
    # $2 is chart ref (e.g. stable/alpha), $7 is dest dir
    # We need to simulate the chart being pulled (which creates a dir named after chart)
    # chart name is the part after /
    CHART_NAME=$(echo "$2" | cut -d/ -f2)
    DEST_DIR="$7"
    mkdir -p "$DEST_DIR/$CHART_NAME"
    exit 0
fi
exit 0
"#,
        log_file_path.display()
    );

    let (temp_dir, new_path) = setup_mock_helm(&script);
    let original_path = env::var("PATH").unwrap_or_default();
    let original_cwd = env::current_dir()?;
    env::set_current_dir(temp_dir.path())?;

    let config_path = temp_dir.path().join("vesshelm.yaml");
    // Define charts in non-alphabetical order
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
    fs::write(&config_path, config_content).unwrap();

    unsafe {
        env::set_var("PATH", &new_path);
    }

    use vesshelm::cli::commands::SyncArgs;
    let args = SyncArgs {
        charts: None,
        ignore_skip: false,
        jobs: 1,
    };

    // Check sync signature:
    // pub async fn run(args: SyncArgs, no_progress: bool, config_path: &Path) -> Result<()>

    let res = tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(vesshelm::cli::commands::sync::run(args, true, &config_path));

    unsafe {
        env::set_var("PATH", original_path);
    }
    env::set_current_dir(original_cwd)?;

    assert!(res.is_ok());

    let log_content = fs::read_to_string(&log_file_path)?;
    let lines: Vec<&str> = log_content.trim().split('\n').collect();

    // We expect alpha, beta, zebra.
    // Chart refs will be "stable/alpha", "stable/beta", "stable/zebra"

    assert_eq!(lines.len(), 3, "Expected 3 sync calls, got: {:?}", lines);
    assert_eq!(lines[0], "stable/alpha");
    assert_eq!(lines[1], "stable/beta");
    assert_eq!(lines[2], "stable/zebra");

    let _ = fs::remove_file(log_file_path);
    Ok(())
}
