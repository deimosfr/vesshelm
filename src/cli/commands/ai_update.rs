use crate::cli::commands::{AiUpdateArgs, sync, update};
use crate::clients::ai;
use crate::config::Config;
use crate::error::VesshelmError;
use crate::util::git::GitUtil;
use crate::util::ui;
use console::style;
use std::fs;
use std::path::Path;

pub async fn run(
    args: AiUpdateArgs,
    no_progress: bool,
    config_path: &Path,
    interaction: &impl crate::util::interaction::UserInteraction,
) -> Result<(), VesshelmError> {
    let config = Config::load_from_path(config_path)?;

    if args.debug_ai {
        // SAFETY: single-threaded CLI startup; setting an env var before spawning the AI
        // subprocess is safe. Read by `cli_subprocess::run_cli`.
        unsafe {
            std::env::set_var(crate::clients::ai::cli_subprocess::DEBUG_ENV, "1");
        }
    }

    let ai_config = config.ai.as_ref().ok_or_else(|| {
        VesshelmError::CommandFailed(
            "AI configuration missing in vesshelm.yaml. Please add an 'ai' section.".to_string(),
        )
    })?;

    // 0. SRE Upgrade Guidelines
    if !no_progress {
        println!("\n{}", style("Upgrade Guidelines").bold().underlined());
        println!(
            " {} Review official release notes and changelogs for the updated versions.",
            style("•").cyan()
        );
        println!(
            " {} Test the upgrade in a staging environment before production deployment.",
            style("•").cyan()
        );
        println!(
            " {} Validate existing configurations against the updated application behavior.",
            style("•").cyan()
        );
        println!(
            " {} Check for new RBAC requirements or security integration changes.",
            style("•").cyan()
        );
        println!();
    }

    // 1. Pre-flight Git Check
    let git = GitUtil::discover(".")?;
    if !args.re_analyze {
        if !no_progress {
            ui::print_step("Checking git status");
        }
        if git.is_dirty()? {
            if args.no_interactive {
                return Err(VesshelmError::CommandFailed(
                    "Local repository has uncommitted changes. Please commit, stash, or use --re-analyze to inspect current changes.".to_string()
                ));
            }
            if !interaction.confirm("Local repository has uncommitted changes. Syncing may produce a messy diff. Proceed anyway?", false)? {
                return Err(VesshelmError::CommandFailed(
                    "Local repository has uncommitted changes. Please commit, stash, or use --re-analyze to inspect current changes.".to_string()
                ));
            }
        }
    }

    // Capture old config to identify what changed, plus the names of charts whose
    // versions were just bumped (used later for the deploy-selection prompt).
    let (old_config, new_config, updated_chart_names) = if args.re_analyze {
        if !no_progress {
            ui::print_step("Loading base configuration from HEAD");
        }

        // Relative path within git repo
        let config_file_name = config_path
            .file_name()
            .and_then(|f| f.to_str())
            .unwrap_or("vesshelm.yaml");

        let old_config_str = git.read_file_from_head(Path::new(config_file_name))?;
        let old: Config = serde_yaml_ng::from_str(&old_config_str).map_err(|e| {
            VesshelmError::CommandFailed(format!("Failed to parse old config from HEAD: {}", e))
        })?;
        let new = Config::load_from_path(config_path)?;
        let mut changed: Vec<String> = new
            .charts
            .iter()
            .filter(|nc| {
                let oc = old
                    .charts
                    .iter()
                    .find(|c| c.name == nc.name && c.namespace == nc.namespace);
                match oc {
                    Some(o) => o.version != nc.version,
                    None => true,
                }
            })
            .map(|c| c.name.clone())
            .collect();
        changed.sort();
        (old, new, changed)
    } else {
        let old = config.clone();

        // 2. Check which charts are outdated
        if !no_progress {
            ui::print_step("Checking for chart updates");
        }

        let outdated = update::check_outdated_charts(
            config_path,
            crate::clients::helm::RealHelmClient::new(),
            args.jobs,
            args.charts.as_deref(),
            no_progress,
        )
        .await?;

        if outdated.is_empty() {
            if !no_progress {
                println!("\nAll checked charts are up to date.");
            }
            return Ok(());
        }

        // 3. Determine which outdated charts to update
        let selected = match select_charts_to_update(
            outdated,
            args.charts.is_some(),
            args.no_interactive,
            interaction,
        )? {
            Some(s) => s,
            None => {
                if !no_progress {
                    println!("No charts selected.");
                }
                return Ok(());
            }
        };

        // 4. Apply selected version bumps + sync those charts
        update::apply_chart_updates(config_path, &selected, no_progress)?;

        let mut selected_names: Vec<String> = selected.iter().map(|c| c.name.clone()).collect();
        selected_names.sort();
        let sync_args = crate::cli::commands::SyncArgs {
            charts: Some(selected_names.clone()),
            ignore_skip: false,
            jobs: args.jobs,
        };
        sync::run(sync_args, true, config_path).await?;

        // Reload config to see new versions
        let new = Config::load_from_path(config_path)?;
        (old, new, selected_names)
    };

    // 3. Capture Diff
    if !no_progress {
        ui::print_step("Capturing Kubernetes template changes (diff)");
    }
    let diff = git.get_diff(args.re_analyze)?;
    if diff.is_empty() {
        println!("{} No changes detected.", style("✔").green());
        return Ok(());
    }

    // 4. Build Overrides Context
    if !no_progress {
        ui::print_step("Extracting technical overrides from vesshelm.yaml");
    }
    let mut overrides_context = String::new();

    for new_chart in &new_config.charts {
        let old_chart = old_config
            .charts
            .iter()
            .find(|c| c.name == new_chart.name && c.namespace == new_chart.namespace);

        let changed = match old_chart {
            Some(old) => old.version != new_chart.version,
            None => true, // New chart added
        };

        if changed {
            overrides_context.push_str(&format!(
                "--- Chart: {} ({}) ---\n",
                new_chart.name, new_chart.namespace
            ));

            if let Some(values) = &new_chart.values
                && let Ok(yaml) = serde_yaml_ng::to_string(values)
            {
                overrides_context.push_str("Inline Values:\n");
                overrides_context.push_str(&yaml);
                overrides_context.push('\n');
            }

            if let Some(files) = &new_chart.values_files {
                for file in files {
                    if let Ok(content) = fs::read_to_string(file) {
                        overrides_context.push_str(&format!("Values File ({}):\n", file));
                        overrides_context.push_str(&content);
                        overrides_context.push('\n');
                    }
                }
            }

            if let Some(append) = &new_chart.helm_args_append {
                overrides_context.push_str(&format!("Helm Args Append: {}\n", append));
            }
            if let Some(over) = &new_chart.helm_args_override {
                overrides_context.push_str(&format!("Helm Args Override: {}\n", over));
            }
            overrides_context.push('\n');
        }
    }

    // 5. AI Analysis
    let ai_client = ai::create_client(ai_config)?;

    // Spinner suppressed when no_progress, or when --debug-ai is on (debug streams
    // child stderr live, the spinner ticks would interleave/garble that output).
    let spinner = if no_progress || args.debug_ai {
        if !no_progress {
            ui::print_step("Analyzing changes...");
        }
        None
    } else {
        let pb = indicatif::ProgressBar::new_spinner();
        pb.set_style(
            indicatif::ProgressStyle::with_template("{spinner:.green} {msg} [{elapsed}]")
                .unwrap_or_else(|_| indicatif::ProgressStyle::default_spinner()),
        );
        pb.set_message("Analyzing changes...");
        pb.enable_steady_tick(std::time::Duration::from_millis(100));
        Some(pb)
    };

    let result = ai_client.analyze_upgrade(&diff, &overrides_context).await;

    if let Some(pb) = spinner {
        pb.finish_and_clear();
    }

    let analysis = result?;

    // 6. Output Summary with colorization
    println!(
        "\n{}",
        style("AI Technical Analysis Report").bold().underlined()
    );
    println!("{}", format_analysis_report(&analysis));

    println!(
        "\n{} Review the technical analysis above carefully before committing changes.",
        style("💡").yellow()
    );

    let can_deploy = !analysis.contains("[ERROR]");

    if can_deploy && !args.no_interactive && !updated_chart_names.is_empty() {
        let defaults = vec![false; updated_chart_names.len()];
        let picks = interaction
            .multi_select(
                "Select the charts you want to deploy",
                &updated_chart_names,
                &defaults,
            )
            .map_err(|e| VesshelmError::CommandFailed(format!("Selection failed: {}", e)))?;
        if picks.is_empty() {
            if !no_progress {
                println!("No charts selected for deploy.");
            }
        } else {
            let deploy_charts: Vec<String> = picks
                .into_iter()
                .map(|i| updated_chart_names[i].clone())
                .collect();
            let deploy_args = crate::cli::commands::DeployArgs {
                dry_run: false,
                charts: Some(deploy_charts),
                no_interactive: true,
                take_ownership: false,
                force: false,
                no_diff: false,
                debug: false,
            };
            crate::cli::commands::deploy::run(deploy_args, no_progress, config_path, interaction)
                .await?;
        }
    }

    Ok(())
}

/// Pick which outdated charts to update.
/// - If `explicit_charts_arg` is true, the caller already filtered with positional args:
///   take the whole list, no prompt.
/// - Else if `no_interactive` is true, take the whole list, no prompt.
/// - Otherwise, prompt the user with a multi-select (all unchecked by default).
///   Returns `Ok(None)` when the user picks nothing.
fn select_charts_to_update(
    outdated: Vec<update::OutdatedChart>,
    explicit_charts_arg: bool,
    no_interactive: bool,
    interaction: &impl crate::util::interaction::UserInteraction,
) -> Result<Option<Vec<update::OutdatedChart>>, VesshelmError> {
    if explicit_charts_arg || no_interactive {
        return Ok(Some(outdated));
    }
    let labels: Vec<String> = outdated
        .iter()
        .map(|c| format!("{} {} -> {}", c.name, c.current, c.latest))
        .collect();
    let defaults = vec![false; outdated.len()];
    let picks = interaction
        .multi_select("Select the charts you want to update", &labels, &defaults)
        .map_err(|e| VesshelmError::CommandFailed(format!("Selection failed: {}", e)))?;
    if picks.is_empty() {
        Ok(None)
    } else {
        Ok(Some(
            picks.into_iter().map(|i| outdated[i].clone()).collect(),
        ))
    }
}

fn format_analysis_report(report: &str) -> String {
    let mut formatted = String::new();
    let mut prev_blank = false;
    for line in report.lines() {
        let is_blank = line.trim().is_empty();
        if is_blank && prev_blank {
            continue;
        }
        prev_blank = is_blank;

        let trimmed = line.trim_start();
        let indent = &line[..line.len() - trimmed.len()];

        let styled_line = if let Some(rest) = trimmed.strip_prefix("[OK]") {
            format!("{}{} {}", indent, style("[OK]").bold().green(), rest)
        } else if let Some(rest) = trimmed.strip_prefix("[WARN]") {
            format!("{}{} {}", indent, style("[WARN]").bold().yellow(), rest)
        } else if let Some(rest) = trimmed.strip_prefix("[ERROR]") {
            format!("{}{} {}", indent, style("[ERROR]").bold().red(), rest)
        } else if trimmed.starts_with("OVERRIDE_IMPACT:") {
            format!(
                "{}{} {}",
                indent,
                style("➜").bold().cyan(),
                style("Override Impacts:").bold().cyan()
            )
        } else if trimmed.starts_with("GENERAL_CHANGE:") {
            format!(
                "{}{} {}",
                indent,
                style("⚙").bold().blue(),
                style("General Changes:").bold().blue()
            )
        } else {
            line.to_string()
        };
        formatted.push_str(&styled_line);
        formatted.push('\n');
    }
    formatted
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util::error::InteractionError;
    use crate::util::interaction::UserInteraction;
    use std::sync::Mutex;

    #[derive(Default)]
    struct StubInteraction {
        multi_select_response: Mutex<Option<Vec<usize>>>,
        multi_select_calls: Mutex<u32>,
    }

    impl StubInteraction {
        fn with_picks(picks: Vec<usize>) -> Self {
            Self {
                multi_select_response: Mutex::new(Some(picks)),
                multi_select_calls: Mutex::new(0),
            }
        }
    }

    impl UserInteraction for StubInteraction {
        fn confirm(&self, _: &str, _: bool) -> Result<bool, InteractionError> {
            Ok(true)
        }
        fn input(&self, _: &str, _: Option<&str>) -> Result<String, InteractionError> {
            Ok(String::new())
        }
        fn select(&self, _: &str, _: &[String], _: usize) -> Result<usize, InteractionError> {
            Ok(0)
        }
        fn fuzzy_select(&self, _: &str, _: &[String], _: usize) -> Result<usize, InteractionError> {
            Ok(0)
        }
        fn multi_select(
            &self,
            _: &str,
            _: &[String],
            _: &[bool],
        ) -> Result<Vec<usize>, InteractionError> {
            *self.multi_select_calls.lock().unwrap() += 1;
            Ok(self
                .multi_select_response
                .lock()
                .unwrap()
                .clone()
                .unwrap_or_default())
        }
    }

    fn sample_outdated() -> Vec<update::OutdatedChart> {
        vec![
            update::OutdatedChart {
                name: "alloy".into(),
                current: "1.7.0".into(),
                latest: "1.8.0".into(),
            },
            update::OutdatedChart {
                name: "bentopdf".into(),
                current: "1.0.15".into(),
                latest: "1.0.16".into(),
            },
        ]
    }

    #[test]
    fn no_interactive_skips_prompt_and_takes_all() {
        let stub = StubInteraction::default();
        let res = select_charts_to_update(sample_outdated(), false, true, &stub).unwrap();
        let picked = res.expect("expected charts");
        assert_eq!(picked.len(), 2);
        assert_eq!(*stub.multi_select_calls.lock().unwrap(), 0);
    }

    #[test]
    fn explicit_charts_arg_skips_prompt() {
        let stub = StubInteraction::default();
        let res = select_charts_to_update(sample_outdated(), true, false, &stub).unwrap();
        let picked = res.expect("expected charts");
        assert_eq!(picked.len(), 2);
        assert_eq!(*stub.multi_select_calls.lock().unwrap(), 0);
    }

    #[test]
    fn interactive_picks_subset() {
        let stub = StubInteraction::with_picks(vec![1]);
        let res = select_charts_to_update(sample_outdated(), false, false, &stub).unwrap();
        let picked = res.expect("expected charts");
        assert_eq!(picked.len(), 1);
        assert_eq!(picked[0].name, "bentopdf");
        assert_eq!(*stub.multi_select_calls.lock().unwrap(), 1);
    }

    #[test]
    fn empty_selection_returns_none() {
        let stub = StubInteraction::with_picks(vec![]);
        let res = select_charts_to_update(sample_outdated(), false, false, &stub).unwrap();
        assert!(res.is_none());
        assert_eq!(*stub.multi_select_calls.lock().unwrap(), 1);
    }
}
