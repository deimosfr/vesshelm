use anyhow::{Context, Result, anyhow};
use colored::*;
use console::style;
use dialoguer::Confirm;
use std::io::Write;
use std::process::Stdio;
use tempfile::NamedTempFile;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use vesshelm::config::{Chart, Config, HelmConfig};

use super::DeployArgs;

pub async fn run(args: DeployArgs, no_progress: bool, config_path: &std::path::Path) -> Result<()> {
    // Load configuration
    let config = Config::load_from_path(config_path)?;

    // Check if helm config is present
    let helm_config = match &config.helm {
        Some(h) => h,
        None => {
            eprintln!(
                "{} No helm configuration found in vesshelm.yaml. Skipping deployment.",
                "⚠️".yellow()
            );
            return Ok(());
        }
    };

    println!("{} 🚀 Starting deployment...", style("==>").bold().green());

    // Process charts
    let mut sorted_charts = vesshelm::util::dag::sort_charts(&config.charts)
        .context("Failed to resolve chart dependencies")?;

    // Filter charts based on args.only
    if let Some(only) = &args.only {
        let available_names: Vec<_> = sorted_charts.iter().map(|c| c.name.as_str()).collect();
        vesshelm::util::filter::validate_only_args(&available_names, only)?;
        sorted_charts.retain(|chart| only.contains(&chart.name));
    }

    let total_charts = sorted_charts.len() as u64;
    let tracker = vesshelm::util::progress::ProgressTracker::new(total_charts, no_progress)
        .context("Failed to initialize progress tracker")?;

    let mut deployed_count = 0;
    let mut failed_count = 0;
    let mut skipped_count = 0;
    let mut ignored_count = 0;

    for chart in sorted_charts {
        if chart.no_deploy {
            tracker.println(&format!(
                " {} {} (no_deploy=true)",
                "⏭ ".yellow(),
                chart.name
            ));
            skipped_count += 1;
            tracker.inc();
            continue;
        }

        match deploy_chart(
            chart,
            &config.destinations,
            helm_config,
            args.dry_run,
            args.no_interactive,
            args.force,
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
                    " {} {} {}: {}",
                    style("[Fail]").red(),
                    "✗", // CROSS emoji not imported here locally properly, using string
                    chart.name,
                    e
                ));
                // Fail fast
                break;
            }
        }
        tracker.inc();
    }

    // ... logic continues ...

    tracker.finish_with_message("Deployment ended");

    println!("\n\n{}", style("Summary:").bold());
    println!("  Deployed: {}", style(deployed_count).green());
    println!("  Failed:   {}", style(failed_count).red());
    println!("  Skipped:  {}", style(skipped_count).yellow());
    println!("  Ignored:  {}", style(ignored_count).dim());

    if failed_count > 0 {
        anyhow::bail!("Deployment failed for some charts");
    }

    Ok(())
}

enum DeployStatus {
    Deployed,
    Skipped,
    Ignored,
}

async fn deploy_chart(
    chart: &Chart,
    destinations: &[vesshelm::config::Destination],
    global_helm_config: &HelmConfig,
    dry_run: bool,
    no_interactive: bool,
    force: bool,
    tracker: &vesshelm::util::progress::ProgressTracker,
) -> Result<DeployStatus> {
    tracker.set_message(format!("Deploying {}...", chart.name));
    tracker.println(&format!(
        "{} Deploying chart {}",
        "📦 ".blue(),
        chart.name.bold()
    ));
    // Determine destination path
    let dest_path = get_destination_path(chart, destinations)?;

    // Construct Helm arguments
    let mut args_str = construct_helm_args(chart, global_helm_config)?;

    // Prepare values flags
    let mut values_flags = String::new();

    // Handle values_files
    if let Some(files) = &chart.values_files {
        for file in files {
            values_flags.push_str(" -f ");
            values_flags.push_str(file);
        }
    }

    // Handle inline values
    let _values_temp_file: Option<NamedTempFile>; // Keep alive until function end
    if let Some(values) = &chart.values {
        let content = vesshelm::util::helm::merge_values(values)?;
        let mut file = tempfile::Builder::new().suffix(".yaml").tempfile()?;
        write!(file, "{}", content)?;
        values_flags.push_str(" -f ");
        values_flags.push_str(
            file.path()
                .to_str()
                .ok_or_else(|| anyhow!("Invalid path for value file"))?,
        );
        _values_temp_file = Some(file);
    } else {
        _values_temp_file = None;
    }

    // Append values to args
    args_str.push_str(&values_flags);

    // Interpolate variables
    let final_args = interpolate_variables(&args_str, chart, &dest_path)?;

    // Handle Diff
    if dry_run || global_helm_config.diff_enabled {
        // If the user didn't provide specific diff args, we might want to default to something sensible.
        // Typically "helm diff upgrade release chart [flags]"
        // Our args_template is "upgrade --install ..." so we can't easily reuse it without parsing.
        // Let's assume the user provides a full command string for diff if they want custom behavior,
        // otherwise we fallback to a simple default or maybe we should reuse the `args` but prepend `diff`?
        // Reuse `args` but prepend `diff` might work if `args` starts with `upgrade`.
        // But `args` is "{{ name }} {{ destination }} ...".
        // `helm upgrade {{ name }} ...` -> `helm diff upgrade {{ name }} ...`?
        // The default `helm.args` is "{{ name }} ...". It misses the verb "upgrade".
        // Wait, let's check `vesshelm.yaml` default.
        // It was: `args: "{{ name }} {{ destination }}/{{ name }} -n {{ namespace }} ..."`
        // The command run was `helm upgrade --install my-chart ...`
        // Oh, the log said: `Running: helm upgrade --install my-chart ...`
        // But `vesshelm.yaml` default `args` I checked in `init.rs` was:
        // `args: "{{ name }} ..."` NO verb.
        // Wait, let me check `init.rs` again.

        // Actually, in `deploy.rs` I see `execute_helm_command` doing `Command::new("helm").args(&parts)`.
        // If `args` is just `{{ name }} ...`, then where does `upgrade --install` come from?
        // Ah, in `vesshelm.yaml` generated by `init` usually it SHOULD include the verb.
        // Let me check my memory of `init.rs` or `vesshelm.yaml` content from previous steps.
        // Step 260: `args: "{{ name }} ..."` - NO verb in default content I verified?
        // BUT Step 172 Walkthrough said: `config: helm: args: "upgrade --install {{ name }} ..."`
        // Step 227 (Init update) I wrote: `args: "{{ name }} {{ destination }} ..."`
        // This looks like I missed the verb in the `init.rs` update step!
        // If I missed the verb, then `vesshelm deploy` would run `helm my-chart ...` which is invalid.
        // BUT verification Step 247 output: `Running: helm upgrade --install my-chart ...` was NOT from `init` test, it was from manual run?
        // No, Step 167 verification output showed `Running: helm upgrade --install ...`.
        // Is it possible I added `upgrade --install` in the `vesshelm.yaml` during manual test but `init.rs` has it missing?
        // Yes, likely.

        // So current `init.rs` has `args: "{{ name }} ..."` (MISSING VERB).
        // I need to fix `init.rs` as well to include `upgrade --install`.

        // Back to diff:
        // `helm diff upgrade` is the command.
        // If `args` includes `upgrade --install`, we can't just prepend `diff`. `diff upgrade` doesn't take `--install`.
        // So `diff_args` should be separate.

        // Use default if None
        let diff_template = global_helm_config.diff_args.as_deref().unwrap_or(
            "diff upgrade --allow-unreleased {{ name }} {{ destination }} -n {{ namespace }}",
        );

        let mut final_diff_args = interpolate_variables(diff_template, chart, &dest_path)?;
        final_diff_args.push_str(&values_flags);

        let diff_output = execute_helm_diff(&final_diff_args, tracker).await?;

        // Check if diff is empty (no changes)
        // Helm diff usually outputs nothing or "No changes" if there are no changes,
        // but sometimes it outputs headers.
        // Let's assume standard behavior: if stdout is empty or contains specific "no changes" message?
        // Actually, helm-diff plugin exit code might be useful?
        // But `helm diff upgrade` returns 0 even if changes.
        // We'll rely on output content.
        let diff_content = String::from_utf8_lossy(&diff_output.stdout);
        if console::strip_ansi_codes(&diff_content).trim().is_empty() {
            if force {
                tracker.println(&format!(
                    "{} No changes detected, but forcing deployment for {}.",
                    "⚠️ ".yellow(),
                    chart.name.bold()
                ));
            } else {
                tracker.println(&format!(
                    "{} No changes for {}. Skipping.",
                    "⏭ ".dimmed(),
                    chart.name.bold()
                ));
                return Ok(DeployStatus::Skipped);
            }
        }

        // Print diff to stdout for user to see
        // Print diff to stdout for user to see
        tracker.println(&diff_content);

        if dry_run {
            return Ok(DeployStatus::Ignored);
        }

        // Interactive Confirmation
        if !no_interactive && !force {
            let confirmation = tracker.suspend(|| {
                println!();
                Confirm::new()
                    .with_prompt(format!(
                        "Do you want to deploy {}?",
                        chart.name.bold().cyan()
                    ))
                    .default(false)
                    .interact()
                    .context("Failed to read user confirmation")
            });
            let confirmation = confirmation?;

            if !confirmation {
                tracker.println(&format!(
                    " {} User skipped deployment of {}.",
                    "⏭ ".yellow(),
                    chart.name.bold()
                ));
                return Ok(DeployStatus::Ignored);
            }
        }
    }

    // Execute Helm command
    execute_helm_command(&final_args, tracker).await?;

    Ok(DeployStatus::Deployed)
}

fn get_destination_path(
    chart: &Chart,
    destinations: &[vesshelm::config::Destination],
) -> Result<String> {
    // 1. Check for destination override in chart
    if let Some(override_path) = &chart.dest {
        if let Some(d) = destinations.iter().find(|d| d.name == *override_path) {
            return Ok(d.path.clone());
        }
        // Fallback? Spec says destination_override refers to destination name.
        return Err(anyhow!(
            "Destination override '{}' not found in destinations",
            override_path
        ));
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

    Err(anyhow!(
        "Could not determine destination path for chart {}",
        chart.name
    ))
}

fn construct_helm_args(chart: &Chart, global_helm_config: &HelmConfig) -> Result<String> {
    if let Some(override_args) = &chart.helm_args_override {
        return Ok(override_args.clone());
    }

    let mut args = global_helm_config.args.clone();

    if let Some(append_args) = &chart.helm_args_append {
        args.push(' ');
        args.push_str(append_args);
    }

    Ok(args)
}

fn interpolate_variables(args_template: &str, chart: &Chart, destination: &str) -> Result<String> {
    let mut result = args_template.to_string();

    // Calculate full chart path for robust replacement
    let full_chart_path = if chart.repo_name.is_none() {
        // For local charts, destination IS the chart path in our logic above
        destination.to_string()
    } else {
        // For remote charts, destination is parent dir
        format!("{}/{}", destination, chart.name)
    };

    // Smart replacement: Handle the common pattern "{{ destination }}/{{ name }}" first
    // This prevents "./my-chart/my-chart" issue for local charts
    result = result.replace("{{ destination }}/{{ name }}", &full_chart_path);
    result = result.replace("{{destination}}/{{name}}", &full_chart_path);

    result = result.replace("{{ name }}", &chart.name);
    result = result.replace("{{ destination }}", destination);
    result = result.replace("{{ namespace }}", &chart.namespace);
    result = result.replace("{{ version }}", chart.version.as_deref().unwrap_or(""));
    result = result.replace("{{ chart_path }}", &full_chart_path);

    // Also support {{name}} without spaces just in case
    result = result.replace("{{name}}", &chart.name);
    result = result.replace("{{destination}}", destination);
    result = result.replace("{{namespace}}", &chart.namespace);
    result = result.replace("{{version}}", chart.version.as_deref().unwrap_or(""));
    result = result.replace("{{chart_path}}", &full_chart_path);

    Ok(result)
}

async fn execute_helm_command(
    args: &str,
    tracker: &vesshelm::util::progress::ProgressTracker,
) -> Result<()> {
    tracker.println(&format!("{} helm {}", "⚙️ ".dimmed(), args.dimmed()));

    let parts: Vec<&str> = args.split_whitespace().collect();

    let mut cmd = Command::new("helm");
    cmd.args(&parts);
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    // We need to group the child process or simply track its PID.
    // In Rust tokio::process::Command, we get a Child handle.

    let mut child = cmd.spawn().context("Failed to execute helm command")?;

    // Capture pid for signal forwarding
    let pid = child
        .id()
        .ok_or_else(|| anyhow!("Failed to get helm process id"))?;

    let stdout = child.stdout.take().context("Failed to capture stdout")?;
    let stderr = child.stderr.take().context("Failed to capture stderr")?;

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
            tracker.println(&format!("{}", format!("Received signal ({}). Stopping helm...", signal_name).yellow()));

            // Terminate child process
            terminate_process(&mut child, pid).await?;

            // Wait for child to exit
            child.wait().await?
        }
    };

    let _ = tokio::join!(task_out, task_err);

    if !status.success() {
        // If we killed it, it likely returned non-zero.
        // If it was a signal interruption, we generally want to return an error unless handled.
        // If we deliberately stopped it, maybe we want to propagate the interruption.
        // For now, failure is failure.
        return Err(anyhow!(
            "Helm command failed/interrupted with status: {}",
            status
        ));
    }

    Ok(())
}

async fn execute_helm_diff(
    args: &str,
    tracker: &vesshelm::util::progress::ProgressTracker,
) -> Result<std::process::Output> {
    tracker.println(&format!("{} helm {}", "🔎 ".dimmed(), args.dimmed()));

    let parts: Vec<&str> = args.split_whitespace().collect();

    // Helm diff outputs to stdout mostly.
    let output = Command::new("helm")
        .args(&parts)
        .env("HELM_DIFF_COLOR", "true")
        .output()
        .await
        .context("Failed to execute helm diff command")?;

    if !output.status.success() {
        // Print stderr if failed
        let stderr = String::from_utf8_lossy(&output.stderr);
        tracker.println(&stderr);
        return Err(anyhow!(
            "Helm diff command failed with status: {}",
            output.status
        ));
    }

    // Convert tokio Output to std Output for compatibility if needed,
    // or just return tokio's Output which is compatible in fields (status, stdout, stderr).
    // The signature says `std::process::Output`. Use `into_std`? No, they are same struct layout usually?
    // Actually `tokio::process::Command::output` returns `std::process::Output`. Yes.

    Ok(output)
}

#[cfg(unix)]
async fn wait_for_signal() -> Result<String> {
    use tokio::signal::unix::{SignalKind, signal};
    let mut sigint = signal(SignalKind::interrupt())?;
    let mut sigterm = signal(SignalKind::terminate())?;
    tokio::select! {
        _ = sigint.recv() => Ok("SIGINT".to_string()),
        _ = sigterm.recv() => Ok("SIGTERM".to_string()),
    }
}

#[cfg(windows)]
async fn wait_for_signal() -> Result<String> {
    tokio::signal::ctrl_c().await?;
    Ok("CTRL+C".to_string())
}

#[cfg(unix)]
async fn terminate_process(_child: &mut tokio::process::Child, pid: u32) -> Result<()> {
    // Try graceful shutdown with SIGTERM first
    let _ = std::process::Command::new("kill")
        .arg("-TERM")
        .arg(pid.to_string())
        .output();
    Ok(())
}

#[cfg(windows)]
async fn terminate_process(child: &mut tokio::process::Child, _pid: u32) -> Result<()> {
    // Windows doesn't support SIGTERM, so we kill the process
    child.kill().await.context("Failed to kill child process")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use vesshelm::config::{Chart, HelmConfig};

    #[test]
    fn test_interpolate_variables() -> Result<()> {
        let chart = Chart {
            name: "my-chart".to_string(),
            repo_name: Some("stable".to_string()),
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
        };
        let dest = "./charts/my-chart";
        let template =
            "upgrade {{ name }} {{ destination }} -n {{ namespace }} --version {{ version }}";

        let result = interpolate_variables(template, &chart, dest)?;
        assert_eq!(
            result,
            "upgrade my-chart ./charts/my-chart -n my-ns --version 1.0.0"
        );
        Ok(())
    }

    #[test]
    fn test_construct_helm_args_override() -> Result<()> {
        let chart = Chart {
            name: "test".to_string(),
            repo_name: Some("test".to_string()),
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
        };
        let global = HelmConfig {
            args: "default".to_string(),
            diff_enabled: false,
            diff_args: None,
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
        };
        let global = HelmConfig {
            args: "default".to_string(),
            diff_enabled: false,
            diff_args: None,
        };

        let result = construct_helm_args(&chart, &global).unwrap();
        assert_eq!(result, "default --extra");
    }
}
