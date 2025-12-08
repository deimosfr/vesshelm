use regex::Regex;
use std::process::Stdio;

use crate::config::{Chart, Config, Destination, VesshelmConfig};
use crate::error::VesshelmError;
use crate::util::progress::ProgressTracker;
use crate::util::{dag, filter};

use console::style;
use dialoguer::Confirm;

use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

use super::DeployArgs;

use crate::util::interaction::UserInteraction;
use crate::util::ui;

pub async fn run(
    args: DeployArgs,
    no_progress: bool,
    config_path: &std::path::Path,
    interaction: &impl UserInteraction,
) -> Result<(), VesshelmError> {
    // Load configuration
    let config = Config::load_from_path(config_path)?;

    // Check if helm config is present
    let helm_config = match &config.vesshelm {
        Some(h) => h,
        None => {
            eprintln!(
                "{}",
                style("No vesshelm configuration found in vesshelm.yaml. Skipping deployment.")
                    .yellow()
            );
            return Ok(());
        }
    };

    // Load variables and secrets
    let mut all_variable_files = Vec::new();
    if let Some(files) = &config.variables_files {
        all_variable_files.extend(files.clone());
    }
    if let Some(files) = &config.secrets_files {
        all_variable_files.extend(files.clone());
    }

    let variable_context = if !all_variable_files.is_empty() {
        let base_path = config_path.parent().unwrap_or(std::path::Path::new("."));
        crate::util::variables::load_variables(&all_variable_files, base_path)
            .await
            .map_err(|e| {
                VesshelmError::CommandFailed(format!("Failed to load variables/secrets: {}", e))
            })?
    } else {
        serde_yaml_ng::Value::Null
    };

    ui::print_step("Starting deployment...");

    // Process charts
    let mut sorted_charts = dag::sort_charts(&config.charts).map_err(VesshelmError::from)?;

    // Filter charts based on args.charts
    let selected_charts = match (args.charts.clone(), args.no_interactive) {
        (Some(charts_args), _) if charts_args.iter().any(|s| s == "all") => {
            // "all" keyword present, deploy all charts
            None
        }
        (Some(charts_args), _) => Some(charts_args),
        (None, true) => {
            // No args and non-interactive: fail safety check
            // We enforce explicit "all" to prevent accidental full deployments in scripts
            return Err(VesshelmError::CommandFailed(
                "No charts specified. Use 'all' to deploy everything or specify chart names."
                    .to_string(),
            ));
        }
        (None, false) => {
            // No args provided - Interactive Selection
            if sorted_charts.is_empty() {
                println!("No charts found in configuration.");
                return Ok(());
            }

            let mut display_charts: Vec<_> = sorted_charts.iter().collect();
            display_charts.sort_by(|a, b| a.name.cmp(&b.name));

            let mut items = vec!["All charts".to_string()];
            for chart in &display_charts {
                items.push(chart.selection_label());
            }

            let selection = interaction.fuzzy_select("Select charts to deploy", &items, 0)?;

            if selection == 0 {
                // Selected "All charts"
                None
            } else {
                // Selected specific chart (index offset by 1)
                // Use display_charts to map selection back to chart name
                Some(vec![display_charts[selection - 1].name.clone()])
            }
        }
    };

    if let Some(charts) = selected_charts {
        let available_names: Vec<_> = sorted_charts.iter().map(|c| c.name.as_str()).collect();
        filter::validate_chart_args(&available_names, &charts)?;
        sorted_charts.retain(|chart| charts.contains(&chart.name));
    }

    let total_charts = sorted_charts.len() as u64;
    let tracker = ProgressTracker::new(total_charts, no_progress)?;

    let mut deployed_count = 0;
    let mut failed_count = 0;
    let mut skipped_count = 0;
    let mut ignored_count = 0;

    for chart in sorted_charts {
        if chart.no_deploy {
            tracker.println(&format!(
                " {} {} (no_deploy=true)",
                style("==>").yellow(),
                chart.name
            ));
            skipped_count += 1;
            tracker.inc();
            continue;
        }

        match deploy_chart(
            chart,
            DeployOptions {
                destinations: &config.destinations,
                global_helm_config: helm_config,
                dry_run: args.dry_run,
                no_interactive: args.no_interactive,
                force: args.force,
                take_ownership: args.take_ownership,
                variable_context: &variable_context,
                no_diff: args.no_diff,
                debug: args.debug,
            },
            &tracker,
        )
        .await
        {
            Ok(DeployStatus::Deployed) => {
                deployed_count += 1;
            }
            Ok(DeployStatus::Skipped) => {
                skipped_count += 1;
            }
            Ok(DeployStatus::Ignored) => {
                ignored_count += 1;
            }
            Err(e) => {
                failed_count += 1;
                tracker.println(&format!(
                    " {} {} {}: {:#}",
                    style("[Fail]").red().bright(),
                    "x",
                    chart.name,
                    e
                ));

                // Fail fast
                break;
            }
        }
        tracker.inc();
    }

    tracker.finish_with_message("Deployment ended");

    crate::util::summary::Summary::new()
        .add("Deployed:", style(deployed_count).green().bright())
        .add("Failed:", style(failed_count).red().bright())
        .add("Skipped:", style(skipped_count).yellow().bright())
        .add("Ignored:", style(ignored_count).dim())
        .print();

    if failed_count > 0 {
        return Err(VesshelmError::CommandFailed(
            "Deployment failed for some charts".to_string(),
        ));
    }

    Ok(())
}

enum DeployStatus {
    Deployed,
    Skipped,
    Ignored,
}

struct DeployOptions<'a> {
    destinations: &'a [Destination],
    global_helm_config: &'a VesshelmConfig,
    dry_run: bool,
    no_interactive: bool,
    force: bool,
    take_ownership: bool,
    variable_context: &'a serde_yaml_ng::Value,
    no_diff: bool,
    debug: bool,
}

async fn deploy_chart(
    chart: &Chart,
    options: DeployOptions<'_>,
    tracker: &ProgressTracker,
) -> Result<DeployStatus, VesshelmError> {
    let DeployOptions {
        destinations,
        global_helm_config,
        dry_run,
        no_interactive,
        force,
        take_ownership,
        variable_context,
        no_diff,
        debug,
    } = options;

    tracker.set_message(format!("Deploying {}...", chart.name));
    tracker.println(&format!(
        "{} Deploying chart {}",
        style("==>").blue(),
        style(&chart.name).bold()
    ));

    // Use TempDir to persist files until the end of the deployment function (Drop cleans up)
    let _rendered_temp_dir = tempfile::Builder::new()
        .prefix("vesshelm-values-")
        .tempdir()
        .map_err(|e| {
            VesshelmError::CommandFailed(format!(
                "Failed to create temporary directory for values files: {}",
                e
            ))
        })?;
    let temp_dir_path = _rendered_temp_dir.path().to_path_buf();

    // Wrap the deployment logic in an inner function/block to catch errors while TempDir is alive
    let result = async {
        // Determine destination path
        let dest_path = get_destination_path(chart, destinations)?;

        // Construct Helm arguments
        let mut args_str = construct_helm_args(chart, global_helm_config)?;

        // Prepare values flags
        let mut values_flags = String::new();

        // Handle local 'values.yaml' interpolation (Implicit default values)
        if !variable_context.is_null()
            && !chart.no_interpolation
            && chart.repo_name.is_none()
            && let Some(chart_src_path) = &chart.chart_path
        {
            let values_path = std::path::Path::new(chart_src_path).join("values.yaml");
            if values_path.exists() {
                match crate::util::variables::render_values_file(&values_path, variable_context).await {
                    Ok(rendered) => {
                        let tmp_path = _rendered_temp_dir.path().join("local-values.yaml");
                        if let Err(e) = std::fs::write(&tmp_path, rendered) {
                            tracker.println(&format!(
                                "{} Failed to write temp values: {}",
                                style("WRN").yellow(),
                                e
                            ));
                        } else if let Some(path_str) = tmp_path.to_str() {
                            values_flags.push_str(" -f ");
                            values_flags.push_str(path_str);
                        }
                    }
                    Err(e) => {
                        return Err(VesshelmError::CommandFailed(format!("Failed to render local values.yaml: {}", e)));
                    }
                }
            }
        }

        if let Some(files) = &chart.values_files {
            for (i, file_path_str) in files.iter().enumerate() {
                let path = std::path::Path::new(file_path_str);
                let file_arg =
                    if !variable_context.is_null() && !chart.no_interpolation && path.exists() {
                        let rendered =
                            crate::util::variables::render_values_file(path, variable_context).await
                                .map_err(|e| VesshelmError::CommandFailed(format!("Failed to render values file {:?}: {}", path, e)))?;
                        let tmp_path = _rendered_temp_dir.path().join(format!("values-{}.yaml", i));
                        std::fs::write(&tmp_path, rendered)?;
                        tmp_path
                            .to_str()
                            .ok_or_else(|| VesshelmError::CommandFailed("Invalid temp path".to_string()))?
                            .to_string()
                    } else {
                        file_path_str.clone()
                    };
                values_flags.push_str(" -f ");
                values_flags.push_str(&file_arg);
            }
        }

        // Handle inline values
        if let Some(values) = &chart.values {
            let mut content = crate::util::helm::merge_values(values)?;

            if !variable_context.is_null() && !chart.no_interpolation {
                content = crate::util::variables::render_string(&content, variable_context)
                    .map_err(|e| VesshelmError::CommandFailed(format!("Failed to render inline values for chart {}: {}", chart.name, e)))?;
            }

            let tmp_path = _rendered_temp_dir.path().join("inline-values.yaml");
            std::fs::write(&tmp_path, content)?;
            values_flags.push_str(" -f ");
            values_flags.push_str(tmp_path.to_str().ok_or_else(|| VesshelmError::CommandFailed("Invalid path".to_string()))?);
        }

        // Append values to args
        args_str.push_str(&values_flags);

        // Apply CLI overrides
        if take_ownership {
            args_str.push_str(" --take-ownership");
        }

        // Apply Debug flag
        if debug {
            args_str.push_str(" --debug");
        }

        // Interpolate variables
        let final_args = interpolate_variables(&args_str, chart, &dest_path)?;

        // Handle Diff
        if dry_run || (global_helm_config.diff_enabled && !no_diff) {
            // Use default if None
            let diff_template = global_helm_config.diff_args.as_deref().unwrap_or(
                "diff upgrade --suppress-secrets --allow-unreleased {{ name }} {{ destination }} -n {{ namespace }}",
            );

            let mut final_diff_args = interpolate_variables(diff_template, chart, &dest_path)?;
            final_diff_args.push_str(&values_flags);

            let diff_output = execute_helm_diff(&final_diff_args, tracker).await?;

            // Check if diff is empty (no changes)
            let diff_content = String::from_utf8_lossy(&diff_output.stdout);
            if console::strip_ansi_codes(&diff_content).trim().is_empty() {
                if force {
                    tracker.println(&format!(
                        "{} No changes detected, but forcing deployment for {}.",
                        style("WRN").yellow(),
                        style(&chart.name).bold()
                    ));
                } else {
                    tracker.println(&format!(
                        "{} No changes for {}. Skipping.",
                        style("==>").dim(),
                        style(&chart.name).bold()
                    ));
                    return Ok(DeployStatus::Skipped);
                }
            }

            // Print diff to stdout for user to see
            tracker.println(&diff_content);

            if dry_run {
                return Ok(DeployStatus::Ignored);
            }

            // Interactive Confirmation
            if !(no_interactive || force) {
                let confirmation = tracker.suspend(|| {
                    println!();
                    Confirm::new()
                        .with_prompt(format!(
                            "Do you want to deploy {}?",
                            style(&chart.name).bold().cyan()
                        ))
                        .default(false)
                        .interact()
                        .map_err(|e| VesshelmError::Interaction(crate::util::error::InteractionError::Io(std::io::Error::other(e))))
                });
                let confirmation = confirmation?;

                if !confirmation {
                    tracker.println(&format!(
                        " {} User skipped deployment of {}.",
                        style("==>").yellow(),
                        style(&chart.name).bold()
                    ));
                    return Ok(DeployStatus::Ignored);
                }
            }

        } else if no_diff && global_helm_config.diff_enabled {
            tracker.println(&format!("{} Diff skipped (--no-diff).", style("==>").dim()));
        }

        // Execute Helm command
        execute_helm_command(&final_args, tracker).await?;

        Ok(DeployStatus::Deployed)
    }
    .await;

    // Handle error pause if needed
    if result.is_err() && global_helm_config.deploy_debug_pause && !no_interactive {
        tracker.println(&format!(
            "\n{}",
            ui::style_step("Deployment failed. Pausing for debug...")
        ));
        // Show temp dir location
        tracker.println(&format!(
            "{} Values files are temporarily located at: {}",
            style("INF").blue(),
            style(temp_dir_path.display()).bold()
        ));
        tracker.println(&format!(
            "\n{} Press {} to continue (files will be deleted)...",
            "👉",
            style("Enter").bold()
        ));

        // Suspend progress bar to allow clean input
        tracker.suspend(|| {
            let mut input = String::new();
            let _ = std::io::stdin().read_line(&mut input);
        });
    }

    result
}

fn get_destination_path(
    chart: &Chart,
    destinations: &[crate::config::Destination],
) -> Result<String, VesshelmError> {
    // 1. Check for destination override in chart
    if let Some(override_path) = &chart.dest {
        if let Some(d) = destinations.iter().find(|d| d.name == *override_path) {
            return Ok(d.path.clone());
        }
        // Use as direct path
        return Ok(override_path.clone());
    }

    // 2. Local chart support
    if chart.repo_name.is_none()
        && let Some(path) = &chart.chart_path
    {
        return Ok(path.clone());
    }

    // Default destination
    if let Some(default_dest) = destinations.iter().find(|d| d.name == "default") {
        return Ok(default_dest.path.clone());
    }

    Err(VesshelmError::CommandFailed(format!(
        "Could not determine destination path for chart {}",
        chart.name
    )))
}

fn construct_helm_args(
    chart: &Chart,
    global_helm_config: &VesshelmConfig,
) -> Result<String, VesshelmError> {
    if let Some(override_args) = &chart.helm_args_override {
        return Ok(override_args.clone());
    }

    let mut args = global_helm_config.helm_args.clone();

    if let Some(append_args) = &chart.helm_args_append {
        args.push(' ');
        args.push_str(append_args);
    }

    Ok(args)
}

use std::sync::LazyLock;

static RE_DEST_NAME: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?x)\{\{\s*destination\s*\}\}/\s*\{\{\s*name\s*\}\}")
        .expect("RE_DEST_NAME: invalid regex")
});
static RE_CHART_PATH: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\{\{\s*chart_path\s*\}\}").expect("RE_CHART_PATH: invalid regex")
});
static RE_NAME: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\{\{\s*name\s*\}\}").expect("RE_NAME: invalid regex"));
static RE_RELEASE_NAME: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\{\{\s*release_name\s*\}\}").expect("RE_RELEASE_NAME: invalid regex")
});
static RE_CHART_NAME: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\{\{\s*chart_name\s*\}\}").expect("RE_CHART_NAME: invalid regex")
});
static RE_DESTINATION: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\{\{\s*destination\s*\}\}").expect("RE_DESTINATION: invalid regex")
});
static RE_NAMESPACE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\{\{\s*namespace\s*\}\}").expect("RE_NAMESPACE: invalid regex"));
static RE_VERSION: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\{\{\s*version\s*\}\}").expect("RE_VERSION: invalid regex"));

fn interpolate_variables(
    args_template: &str,
    chart: &Chart,
    destination: &str,
) -> Result<String, VesshelmError> {
    let mut result = args_template.to_string();

    // Calculate full chart path
    let full_chart_path = if chart.repo_name.is_none() {
        // For local charts, destination IS the chart path in our logic above
        destination.to_string()
    } else {
        // For remote charts, destination is parent dir
        format!("{}/{}", destination, chart.name)
    };

    let release_name = chart.release_name();

    // 1. Complex path patterns first ({{ destination }}/{{ name }})
    result = RE_DEST_NAME
        .replace_all(&result, full_chart_path.as_str())
        .to_string();

    // 2. Explicit Path Variable
    result = RE_CHART_PATH
        .replace_all(&result, full_chart_path.as_str())
        .to_string();

    // 3. Release Name (handles {{ name }} and {{ release_name }})
    result = RE_NAME
        .replace_all(&result, release_name.as_str())
        .to_string();
    result = RE_RELEASE_NAME
        .replace_all(&result, release_name.as_str())
        .to_string();

    // 4. Chart Name (Directory/Logical name)
    result = RE_CHART_NAME
        .replace_all(&result, chart.name.as_str())
        .to_string();

    // 5. Other variables
    result = RE_DESTINATION.replace_all(&result, destination).to_string();
    result = RE_NAMESPACE
        .replace_all(&result, chart.namespace.as_str())
        .to_string();
    result = RE_VERSION
        .replace_all(&result, chart.version.as_deref().unwrap_or(""))
        .to_string();

    Ok(result)
}

async fn execute_helm_command(
    args: &str,
    tracker: &crate::util::progress::ProgressTracker,
) -> Result<(), VesshelmError> {
    tracker.println(&format!(
        "{} helm {}",
        style("==>").dim(),
        style(args).dim()
    ));

    let parts: Vec<&str> = args.split_whitespace().collect();

    let mut cmd = Command::new("helm");
    cmd.args(&parts);
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    // We need to group the child process or simply track its PID.
    // In Rust tokio::process::Command, we get a Child handle.

    let mut child = cmd.spawn().map_err(|e| {
        VesshelmError::CommandFailed(format!("Failed to execute helm command: {}", e))
    })?;

    // Capture pid for signal forwarding
    let pid = child
        .id()
        .ok_or_else(|| VesshelmError::CommandFailed("Failed to get helm process id".to_string()))?;

    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| VesshelmError::CommandFailed("Failed to capture stdout".to_string()))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| VesshelmError::CommandFailed("Failed to capture stderr".to_string()))?;

    let mut stdout_reader = BufReader::new(stdout).lines();
    let mut stderr_reader = BufReader::new(stderr).lines();

    let tracker_out = tracker.clone();
    let task_out = tokio::spawn(async move {
        while let Ok(Some(line)) = stdout_reader.next_line().await {
            tracker_out.println(&line);
        }
    });

    let tracker_err = tracker.clone();
    let task_err = tokio::spawn(async move {
        while let Ok(Some(line)) = stderr_reader.next_line().await {
            tracker_err.println(&line);
        }
    });

    // Handle signals
    let status = tokio::select! {
        res = child.wait() => res?,
        signal_res = wait_for_signal() => {
            let signal_name = signal_res?;
            tracker.println(&format!("{}", style(format!("Received signal ({}). Propagating to helm...", signal_name)).yellow().bright()));

            // Terminate child process with the specific signal
            terminate_process(&mut child, pid, &signal_name).await?;

            // Wait for child to exit with timeout
            match tokio::time::timeout(std::time::Duration::from_secs(30), child.wait()).await {
                Ok(res) => res?,
                Err(_) => {
                    tracker.println(&format!("{}", style("Helm process did not exit in time. Force killing...").red()));
                    child.kill().await.map_err(|e| VesshelmError::CommandFailed(format!("Failed to force kill helm: {}", e)))?;
                    child.wait().await?
                }
            }
        }
    };

    let _ = tokio::join!(task_out, task_err);

    if !status.success() {
        return Err(VesshelmError::CommandFailed(format!(
            "Helm command failed/interrupted with status: {}",
            status
        )));
    }

    Ok(())
}

async fn execute_helm_diff(
    args: &str,
    tracker: &crate::util::progress::ProgressTracker,
) -> Result<std::process::Output, VesshelmError> {
    tracker.println(&format!(
        "{} helm {}",
        style("==>").dim(),
        style(args).dim()
    ));

    let parts: Vec<&str> = args.split_whitespace().collect();

    // Helm diff outputs to stdout mostly.
    let mut cmd = Command::new("helm");
    cmd.args(&parts);
    cmd.env("HELM_DIFF_COLOR", "true");
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());
    cmd.kill_on_drop(true);

    let child = cmd.spawn().map_err(|e| {
        VesshelmError::CommandFailed(format!("Failed to execute helm diff command: {}", e))
    })?;

    // Spawn a warning task
    let tracker_clone = tracker.clone();
    let warning_task = tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        tracker_clone.println(&format!(
            "{} Helm diff is taking longer than expected...",
            style("WRN").yellow()
        ));
    });

    // Execute with timeout
    let output_res =
        tokio::time::timeout(std::time::Duration::from_secs(30), child.wait_with_output()).await;

    // Abort the warning task if it hasn't fired yet
    warning_task.abort();

    let output = match output_res {
        Ok(Ok(output)) => output,
        Ok(Err(e)) => {
            return Err(VesshelmError::CommandFailed(format!(
                "Failed to wait for helm diff: {}",
                e
            )));
        }
        Err(_) => {
            tracker.println(&format!(
                "{} Helm diff timed out after 30s. Killing process...",
                style("ERR").red()
            ));
            // Process is killed automatically on drop due to kill_on_drop(true)
            return Err(VesshelmError::CommandFailed(
                "Helm diff command timed out".to_string(),
            ));
        }
    };

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        tracker.println(&stderr);
        return Err(VesshelmError::CommandFailed(format!(
            "Helm diff command failed with status: {}",
            output.status
        )));
    }

    Ok(output)
}

#[cfg(unix)]
async fn wait_for_signal() -> Result<String, VesshelmError> {
    use tokio::signal::unix::{SignalKind, signal};
    let mut sigint = signal(SignalKind::interrupt())?;
    let mut sigterm = signal(SignalKind::terminate())?;
    tokio::select! {
        _ = sigint.recv() => Ok("SIGINT".to_string()),
        _ = sigterm.recv() => Ok("SIGTERM".to_string()),
    }
}

#[cfg(windows)]
async fn wait_for_signal() -> Result<String, VesshelmError> {
    tokio::signal::ctrl_c().await?; // Map IO?
    Ok("CTRL+C".to_string())
}

#[cfg(unix)]
async fn terminate_process(
    _child: &mut tokio::process::Child,
    pid: u32,
    signal_name: &str,
) -> Result<(), VesshelmError> {
    use nix::sys::signal::{self, Signal};
    use nix::unistd::Pid;

    let sig = match signal_name {
        "SIGINT" => Signal::SIGINT,
        _ => Signal::SIGTERM,
    };

    signal::kill(Pid::from_raw(pid as i32), sig).map_err(|e| {
        VesshelmError::CommandFailed(format!(
            "Failed to send signal {} to helm: {}",
            signal_name, e
        ))
    })?;
    Ok(())
}

#[cfg(windows)]
async fn terminate_process(
    child: &mut tokio::process::Child,
    _pid: u32,
    _signal_name: &str,
) -> Result<(), VesshelmError> {
    // Windows doesn't support SIGTERM/SIGINT propagation easily to non-console attached processes via API
    // but Ctrl+C might already be propagated if they share console.
    child.kill().await.map_err(|e| {
        VesshelmError::CommandFailed(format!("Failed to kill child process: {}", e))
    })?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Chart, VesshelmConfig};

    #[test]
    fn test_interpolate_variables() -> anyhow::Result<()> {
        let chart = Chart {
            name: "my-chart".to_string(),
            repo_name: Some("stable".to_string()),
            release_name: Some("my-release".to_string()),
            version: Some("1.0.0".to_string()),
            namespace: "my-ns".to_string(),
            dest: None,
            chart_path: None,
            no_sync: false,
            no_deploy: false,
            comment: None,
            values_files: None,
            helm_args_append: None,
            helm_args_override: None,
            values: None,
            depends: None,
            no_interpolation: false,
        };
        let dest = "./charts/my-chart";
        // {{ name }} should be release name ("my-release")
        // {{ destination }} is dest
        let template =
            "upgrade {{ name }} {{ destination }} -n {{ namespace }} --version {{ version }}";

        let result = interpolate_variables(template, &chart, dest)?;
        assert_eq!(
            result,
            "upgrade my-release ./charts/my-chart -n my-ns --version 1.0.0"
        );

        // Test explicit variables
        let template_explicit = "echo {{ release_name }} {{ chart_name }}";
        let result_explicit = interpolate_variables(template_explicit, &chart, dest)?;
        assert_eq!(result_explicit, "echo my-release my-chart");

        Ok(())
    }

    #[test]
    fn test_construct_helm_args_override() -> anyhow::Result<()> {
        let chart = Chart {
            name: "test".to_string(),
            repo_name: Some("test".to_string()),
            release_name: None,
            version: Some("1.0.0".to_string()),
            namespace: "default".to_string(),
            dest: None,
            chart_path: None,
            no_sync: false,
            no_deploy: false,
            comment: None,
            values_files: None,
            helm_args_append: Some("--ignore-this".to_string()),
            helm_args_override: Some("override".to_string()),
            values: None,
            depends: None,
            no_interpolation: false,
        };
        let global = VesshelmConfig {
            helm_args: "default".to_string(),
            diff_enabled: false,
            diff_args: None,
            deploy_debug_pause: true,
            repo_update_cache_minutes: 60,
        };

        let result = construct_helm_args(&chart, &global).unwrap();
        assert_eq!(result, "override");
        Ok(())
    }

    #[test]
    fn test_construct_helm_args_append() {
        let chart = Chart {
            name: "test".to_string(),
            repo_name: Some("test".to_string()),
            release_name: None,
            version: Some("1.0.0".to_string()),
            namespace: "default".to_string(),
            dest: None,
            chart_path: None,
            no_sync: false,
            no_deploy: false,
            comment: None,
            values_files: None,
            helm_args_append: Some("--extra".to_string()),
            helm_args_override: None,
            values: None,
            depends: None,
            no_interpolation: false,
        };
        let global = VesshelmConfig {
            helm_args: "default".to_string(),
            diff_enabled: false,
            diff_args: None,
            deploy_debug_pause: true,
            repo_update_cache_minutes: 60,
        };

        let result = construct_helm_args(&chart, &global).unwrap();
        assert_eq!(result, "default --extra");
    }

    #[test]
    fn test_take_ownership_flag_logic() {
        let mut args = "upgrade my-release".to_string();
        let take_ownership = true;

        if take_ownership {
            args.push_str(" --take-ownership");
        }

        assert_eq!(args, "upgrade my-release --take-ownership");
    }
}
