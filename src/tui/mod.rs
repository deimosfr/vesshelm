//! Interactive terminal dashboard for Vesshelm.
//!
//! The dashboard intentionally launches the existing Vesshelm subcommands as
//! child processes. This keeps command-line behavior and automation stable,
//! while allowing the UI to stream their output and control their lifecycle.

use std::{
    collections::{HashMap, HashSet, VecDeque},
    ffi::OsString,
    path::{Path, PathBuf},
    process::Stdio,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{
        Block, Borders, Cell, Clear, List, ListItem, Paragraph, Row, Table, TableState, Wrap,
    },
};
use serde::Deserialize;
use tokio::{
    io::{AsyncBufReadExt, AsyncRead, BufReader},
    process::{Child, Command},
    sync::mpsc,
};

use crate::{
    config::{Chart, Config},
    error::VesshelmError,
    lock::Lockfile,
    util::config_updater::ConfigUpdater,
};

const MAX_LOG_LINES: usize = 2_000;
const MAX_ACTIVITY: usize = 12;
const MAX_NAVIGATION_EVENTS_PER_FRAME: usize = 8;
const POD_REFRESH_INTERVAL: Duration = Duration::from_secs(1);

/// Starts the full-screen Vesshelm dashboard.
pub async fn run(config_path: &Path) -> Result<(), VesshelmError> {
    let mut terminal = ratatui::try_init()?;
    let result = App::new(config_path.to_path_buf()).run(&mut terminal).await;
    let restore_result = ratatui::try_restore();

    match (result, restore_result) {
        (Err(error), _) => Err(error),
        (Ok(()), Err(error)) => Err(VesshelmError::Io(error)),
        (Ok(()), Ok(())) => Ok(()),
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum RunStatus {
    Idle,
    Running,
    Succeeded,
    Failed,
    Canceled,
}

#[derive(Debug, Eq, PartialEq)]
enum SyncState {
    NotApplicable,
    Synced,
    Pending(String),
}

impl SyncState {
    fn label(&self) -> &str {
        match self {
            Self::NotApplicable => "—",
            Self::Synced => "synced",
            Self::Pending(version) => version,
        }
    }

    fn style(&self) -> Style {
        match self {
            Self::NotApplicable => Style::default(),
            Self::Synced => Style::new().fg(Color::Green),
            Self::Pending(_) => Style::new().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        }
    }
}

impl RunStatus {
    fn label(self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Running => "running",
            Self::Succeeded => "ok",
            Self::Failed => "failed",
            Self::Canceled => "canceled",
        }
    }

    fn color(self) -> Color {
        match self {
            Self::Idle => Color::DarkGray,
            Self::Running => Color::Yellow,
            Self::Succeeded => Color::Green,
            Self::Failed => Color::Red,
            Self::Canceled => Color::LightRed,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ActionKind {
    Sync,
    Deploy,
    Validate,
    Add,
    Delete,
    Update,
    AiUpdate,
}

impl ActionKind {
    fn label(self) -> &'static str {
        match self {
            Self::Sync => "sync",
            Self::Deploy => "deploy",
            Self::Validate => "validate",
            Self::Add => "add",
            Self::Delete => "delete",
            Self::Update => "update",
            Self::AiUpdate => "ai-update",
        }
    }
}

#[derive(Clone, Debug)]
struct Action {
    kind: ActionKind,
    scope: ActionScope,
    options: ActionOptions,
}

#[derive(Clone, Debug)]
enum ActionScope {
    Charts(Vec<String>),
    All,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum UpdateMode {
    Check,
    Apply,
    ApplySync,
}

#[derive(Clone, Debug)]
enum ActionOptions {
    Default,
    Sync {
        ignore_skip: bool,
        jobs: usize,
    },
    Deploy {
        dry_run: bool,
        take_ownership: bool,
        force: bool,
        no_diff: bool,
        debug: bool,
    },
    Update {
        mode: UpdateMode,
        jobs: usize,
    },
    AiUpdate {
        re_analyze: bool,
        jobs: usize,
        debug_ai: bool,
    },
}

impl ActionOptions {
    fn for_kind(kind: ActionKind) -> Self {
        match kind {
            ActionKind::Sync => Self::Sync {
                ignore_skip: false,
                jobs: 5,
            },
            ActionKind::Deploy => Self::Deploy {
                dry_run: true,
                take_ownership: false,
                force: false,
                no_diff: false,
                debug: false,
            },
            ActionKind::Update => Self::Update {
                mode: UpdateMode::Check,
                jobs: 5,
            },
            ActionKind::AiUpdate => Self::AiUpdate {
                re_analyze: false,
                jobs: 5,
                debug_ai: false,
            },
            ActionKind::Validate | ActionKind::Add | ActionKind::Delete => Self::Default,
        }
    }

    fn fields(&self) -> Vec<(&'static str, String)> {
        match self {
            Self::Default => Vec::new(),
            Self::Sync { ignore_skip, jobs } => vec![
                ("Ignore skip conditions", enabled(*ignore_skip)),
                ("Concurrent jobs", jobs.to_string()),
            ],
            Self::Deploy {
                dry_run,
                take_ownership,
                force,
                no_diff,
                debug,
            } => vec![
                ("Dry run", enabled(*dry_run)),
                ("Take ownership", enabled(*take_ownership)),
                ("Force deployment", enabled(*force)),
                ("Disable diff", enabled(*no_diff)),
                ("Helm debug output", enabled(*debug)),
            ],
            Self::Update { mode, jobs } => vec![
                ("Check only (dry run)", enabled(*mode == UpdateMode::Check)),
                ("Apply version updates", enabled(*mode == UpdateMode::Apply)),
                ("Apply and sync", enabled(*mode == UpdateMode::ApplySync)),
                ("Concurrent jobs", jobs.to_string()),
            ],
            Self::AiUpdate {
                re_analyze,
                jobs,
                debug_ai,
            } => vec![
                ("Re-analyze local changes", enabled(*re_analyze)),
                ("Concurrent jobs", jobs.to_string()),
                ("AI provider debug", enabled(*debug_ai)),
            ],
        }
    }

    fn toggle(&mut self, field: usize) {
        match self {
            Self::Sync { ignore_skip, .. } if field == 0 => *ignore_skip = !*ignore_skip,
            Self::Deploy {
                dry_run,
                take_ownership,
                force,
                no_diff,
                debug,
            } => match field {
                0 => {
                    *dry_run = !*dry_run;
                    if *dry_run {
                        *force = false;
                        *no_diff = false;
                    }
                }
                1 => *take_ownership = !*take_ownership,
                2 => {
                    *force = !*force;
                    if *force {
                        *dry_run = false;
                    }
                }
                3 => {
                    *no_diff = !*no_diff;
                    if *no_diff {
                        *dry_run = false;
                    }
                }
                4 => *debug = !*debug,
                _ => {}
            },
            Self::Update { mode, .. } => match field {
                0 => *mode = UpdateMode::Check,
                1 => *mode = UpdateMode::Apply,
                2 => *mode = UpdateMode::ApplySync,
                _ => {}
            },
            Self::AiUpdate {
                re_analyze,
                debug_ai,
                ..
            } => match field {
                0 => *re_analyze = !*re_analyze,
                2 => *debug_ai = !*debug_ai,
                _ => {}
            },
            _ => {}
        }
    }

    fn adjust_jobs(&mut self, field: usize, increase: bool) {
        let jobs = match self {
            Self::Sync { jobs, .. } if field == 1 => Some(jobs),
            Self::Update { jobs, .. } if field == 3 => Some(jobs),
            Self::AiUpdate { jobs, .. } if field == 1 => Some(jobs),
            _ => None,
        };
        if let Some(jobs) = jobs {
            if increase {
                *jobs = jobs.saturating_add(1);
            } else {
                *jobs = jobs.saturating_sub(1).max(1);
            }
        }
    }
}

fn enabled(value: bool) -> String {
    if value {
        "on".to_string()
    } else {
        "off".to_string()
    }
}

impl Action {
    fn charts(kind: ActionKind, charts: Vec<String>) -> Self {
        Self {
            kind,
            scope: ActionScope::Charts(charts),
            options: ActionOptions::for_kind(kind),
        }
    }

    fn all(kind: ActionKind) -> Self {
        Self {
            kind,
            scope: ActionScope::All,
            options: ActionOptions::for_kind(kind),
        }
    }

    fn label(&self) -> String {
        match &self.scope {
            ActionScope::Charts(charts) if charts.len() == 1 => {
                format!("{} {}", self.kind.label(), charts[0])
            }
            ActionScope::Charts(charts) => {
                format!("{} {} selected charts", self.kind.label(), charts.len())
            }
            ActionScope::All if self.kind == ActionKind::Deploy => "deploy all charts".to_string(),
            ActionScope::All if self.kind == ActionKind::Sync => "sync all charts".to_string(),
            ActionScope::All if self.kind == ActionKind::Update => "update all charts".to_string(),
            ActionScope::All if self.kind == ActionKind::AiUpdate => {
                "AI update all charts".to_string()
            }
            ActionScope::All if self.kind == ActionKind::Add => "add a chart".to_string(),
            ActionScope::All => self.kind.label().to_string(),
        }
    }
}

#[derive(Clone, Debug)]
struct OptionEditor {
    action: Action,
    selected: usize,
}

#[derive(Clone, Debug)]
struct VersionEditor {
    chart_name: String,
    namespace: String,
    version: String,
}

impl OptionEditor {
    fn new(action: Action) -> Self {
        Self {
            action,
            selected: 0,
        }
    }

    fn fields(&self) -> Vec<(&'static str, String)> {
        self.action.options.fields()
    }

    fn select_next(&mut self) {
        let count = self.fields().len();
        self.selected = (self.selected + 1) % count;
    }

    fn select_previous(&mut self) {
        let count = self.fields().len();
        self.selected = if self.selected == 0 {
            count - 1
        } else {
            self.selected - 1
        };
    }
}

#[derive(Clone, Debug)]
struct Activity {
    label: String,
    status: RunStatus,
}

#[derive(Clone, Debug)]
struct PodTarget {
    chart: String,
    namespace: String,
    release: String,
    selectors: Vec<HashMap<String, String>>,
    selectors_resolved: bool,
}

#[derive(Clone, Debug)]
struct TrackedPod {
    chart: String,
    name: String,
    age: String,
    phase: String,
    ready: String,
    restarts: u32,
}

#[derive(Debug)]
struct PodUpdate {
    pods: Vec<TrackedPod>,
    error: Option<String>,
    targets: Vec<PodTarget>,
}

#[derive(Deserialize)]
struct KubernetesPodList {
    #[serde(default)]
    items: Vec<KubernetesPod>,
}

#[derive(Deserialize)]
struct KubernetesPod {
    metadata: KubernetesPodMetadata,
    #[serde(default)]
    status: KubernetesPodStatus,
}

#[derive(Default, Deserialize)]
struct KubernetesPodMetadata {
    #[serde(default)]
    name: String,
    #[serde(default)]
    labels: HashMap<String, String>,
    #[serde(default, rename = "creationTimestamp")]
    creation_timestamp: Option<String>,
}

#[derive(Default, Deserialize)]
struct KubernetesPodStatus {
    phase: Option<String>,
    #[serde(default, rename = "containerStatuses")]
    container_statuses: Vec<KubernetesContainerStatus>,
}

#[derive(Deserialize)]
struct KubernetesContainerStatus {
    ready: bool,
    #[serde(rename = "restartCount")]
    restart_count: u32,
}

#[derive(Clone, Debug)]
struct LogLine {
    spans: Vec<LogSpan>,
}

#[derive(Clone, Debug)]
struct LogSpan {
    content: String,
    style: Style,
}

impl LogLine {
    fn to_line(&self) -> Line<'static> {
        Line::from(
            self.spans
                .iter()
                .map(|span| Span::styled(span.content.clone(), span.style))
                .collect::<Vec<_>>(),
        )
    }
}

#[derive(Clone, Debug)]
enum Confirmation {
    Action(Action),
    Quit,
}

impl Confirmation {
    fn message(&self) -> String {
        match self {
            Self::Action(action) => format!("Run `{}`?", action.label()),
            Self::Quit => "An action is still running. Leave the dashboard?".to_string(),
        }
    }
}

struct App {
    config_path: PathBuf,
    charts: Vec<Chart>,
    chart_state: TableState,
    selected_charts: HashSet<String>,
    chart_status: HashMap<String, RunStatus>,
    available_versions: HashMap<String, String>,
    synced_versions: HashMap<String, HashMap<String, String>>,
    activity: VecDeque<Activity>,
    logs: VecDeque<LogLine>,
    log_scroll: u16,
    child: Option<Child>,
    active_action: Option<Action>,
    queued_actions: VecDeque<Action>,
    interactive_action: Option<Action>,
    output_rx: Option<mpsc::UnboundedReceiver<String>>,
    pod_rx: Option<mpsc::UnboundedReceiver<PodUpdate>>,
    pod_tx: Option<mpsc::UnboundedSender<PodUpdate>>,
    pod_targets: Vec<PodTarget>,
    pods: Vec<TrackedPod>,
    pod_error: Option<String>,
    pod_refresh_in_flight: bool,
    next_pod_refresh: Instant,
    cancellation_requested: bool,
    confirmation: Option<Confirmation>,
    option_editor: Option<OptionEditor>,
    version_editor: Option<VersionEditor>,
    show_help: bool,
    fullscreen_logs: bool,
    search_query: String,
    search_editing: bool,
    config_error: Option<String>,
}

impl App {
    fn new(config_path: PathBuf) -> Self {
        let mut app = Self {
            config_path,
            charts: Vec::new(),
            chart_state: TableState::default(),
            selected_charts: HashSet::new(),
            chart_status: HashMap::new(),
            available_versions: HashMap::new(),
            synced_versions: HashMap::new(),
            activity: VecDeque::new(),
            logs: VecDeque::new(),
            log_scroll: 0,
            child: None,
            active_action: None,
            queued_actions: VecDeque::new(),
            interactive_action: None,
            output_rx: None,
            pod_rx: None,
            pod_tx: None,
            pod_targets: Vec::new(),
            pods: Vec::new(),
            pod_error: None,
            pod_refresh_in_flight: false,
            next_pod_refresh: Instant::now(),
            cancellation_requested: false,
            confirmation: None,
            option_editor: None,
            version_editor: None,
            show_help: false,
            fullscreen_logs: false,
            search_query: String::new(),
            search_editing: false,
            config_error: None,
        };
        app.reload_config();
        app
    }

    async fn run(&mut self, terminal: &mut ratatui::DefaultTerminal) -> Result<(), VesshelmError> {
        loop {
            self.drain_output();
            self.drain_pod_updates();
            self.observe_child()?;
            self.refresh_pods();
            terminal.draw(|frame| self.draw(frame))?;

            if event::poll(Duration::from_millis(80))? && self.handle_pending_keys()? {
                return Ok(());
            }

            if let Some(action) = self.interactive_action.take() {
                self.run_interactive_action(terminal, action).await?;
            }

            // Let the stdout/stderr forwarding tasks make progress even when
            // the terminal receives no input.
            tokio::task::yield_now().await;
        }
    }

    fn reload_config(&mut self) {
        match Config::load_from_path(&self.config_path) {
            Ok(mut config) => {
                config
                    .charts
                    .sort_by(|left, right| left.name.cmp(&right.name));
                self.charts = config.charts;
                self.synced_versions =
                    Lockfile::load(&self.config_path.with_file_name("vesshelm.lock"))
                        .unwrap_or_default()
                        .charts
                        .into_iter()
                        .fold(HashMap::new(), |mut versions, chart| {
                            versions
                                .entry(chart.name)
                                .or_insert_with(HashMap::new)
                                .insert(chart.repo_name, chart.version);
                            versions
                        });
                self.selected_charts
                    .retain(|name| self.charts.iter().any(|chart| &chart.name == name));
                self.available_versions.retain(|name, latest| {
                    self.charts
                        .iter()
                        .find(|chart| chart.name == *name)
                        .is_some_and(|chart| {
                            chart
                                .version
                                .as_ref()
                                .is_none_or(|current| current != latest)
                        })
                });
                self.config_error = None;
                if self.charts.is_empty() {
                    self.chart_state.select(None);
                    self.log("No chart found in the configuration.");
                } else {
                    self.reset_visible_selection();
                    self.log(format!("Loaded {} chart(s).", self.charts.len()));
                }
            }
            Err(error) => {
                self.charts.clear();
                self.synced_versions.clear();
                self.chart_state.select(None);
                let message = format!("Cannot load {}: {error}", self.config_path.display());
                self.config_error = Some(message.clone());
                self.log(message);
            }
        }
    }

    fn handle_key(&mut self, key: KeyEvent) -> Result<bool, VesshelmError> {
        if self.show_help {
            self.show_help = false;
            return Ok(false);
        }

        if let Some(confirmation) = self.confirmation.take() {
            match key.code {
                KeyCode::Enter | KeyCode::Char('y') => match confirmation {
                    Confirmation::Action(action) => self.start_action(action)?,
                    Confirmation::Quit => return Ok(true),
                },
                KeyCode::Esc | KeyCode::Char('n') => self.log("Action canceled."),
                _ => self.confirmation = Some(confirmation),
            }
            return Ok(false);
        }

        if let Some(mut editor) = self.option_editor.take() {
            let close_editor = match key.code {
                KeyCode::Down | KeyCode::Char('j') => {
                    editor.select_next();
                    false
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    editor.select_previous();
                    false
                }
                KeyCode::Char(' ') => {
                    editor.action.options.toggle(editor.selected);
                    false
                }
                KeyCode::Char('+') | KeyCode::Right => {
                    editor.action.options.adjust_jobs(editor.selected, true);
                    false
                }
                KeyCode::Char('-') | KeyCode::Left => {
                    editor.action.options.adjust_jobs(editor.selected, false);
                    false
                }
                KeyCode::Enter => {
                    self.start_action(editor.action.clone())?;
                    true
                }
                KeyCode::Esc => {
                    self.log("Action setup canceled.");
                    true
                }
                _ => false,
            };
            if !close_editor {
                self.option_editor = Some(editor);
            }
            return Ok(false);
        }

        if let Some(mut editor) = self.version_editor.take() {
            let close_editor = match key.code {
                KeyCode::Enter => {
                    self.save_chart_version(&editor);
                    true
                }
                KeyCode::Esc => {
                    self.log("Version change canceled.");
                    true
                }
                KeyCode::Backspace => {
                    editor.version.pop();
                    false
                }
                KeyCode::Char(character) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                    editor.version.push(character);
                    false
                }
                _ => false,
            };
            if !close_editor {
                self.version_editor = Some(editor);
            }
            return Ok(false);
        }

        if self.search_editing {
            self.handle_search_key(key);
            return Ok(false);
        }

        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            if self.child.is_some() {
                self.confirmation = Some(Confirmation::Quit);
                return Ok(false);
            }
            return Ok(true);
        }

        if self.fullscreen_logs {
            match key.code {
                KeyCode::Esc | KeyCode::Char('l') | KeyCode::Char('q') => {
                    self.fullscreen_logs = false
                }
                KeyCode::PageUp => self.log_scroll = self.log_scroll.saturating_add(16),
                KeyCode::PageDown => self.log_scroll = self.log_scroll.saturating_sub(16),
                KeyCode::Char('g') => self.log_scroll = 0,
                KeyCode::Char('?') | KeyCode::Char('h') => self.show_help = true,
                _ => {}
            }
            return Ok(false);
        }

        match key.code {
            KeyCode::Char('/') => self.search_editing = true,
            KeyCode::Esc if !self.search_query.is_empty() => self.clear_search(),
            KeyCode::Char('q') => {
                if self.child.is_some() {
                    self.confirmation = Some(Confirmation::Quit);
                } else {
                    return Ok(true);
                }
            }
            KeyCode::Char('?') | KeyCode::Char('h') => self.show_help = true,
            KeyCode::Down | KeyCode::Char('j') => self.select_next(),
            KeyCode::Up | KeyCode::Char('k') => self.select_previous(),
            KeyCode::Char(' ') => self.toggle_selected_chart(),
            KeyCode::Char('a') => self.toggle_all_charts(),
            KeyCode::PageUp => self.log_scroll = self.log_scroll.saturating_add(8),
            KeyCode::PageDown => self.log_scroll = self.log_scroll.saturating_sub(8),
            KeyCode::Char('g') => self.log_scroll = 0,
            KeyCode::Char('l') => self.fullscreen_logs = true,
            KeyCode::Char('r') => self.reload_config(),
            KeyCode::Char('e') => self.edit_selected_version(),
            KeyCode::Char('s') => self.configure_selected(ActionKind::Sync),
            KeyCode::Char('S') => self.configure_all(ActionKind::Sync),
            KeyCode::Char('d') => self.configure_selected(ActionKind::Deploy),
            KeyCode::Char('D') => self.configure_all(ActionKind::Deploy),
            KeyCode::Char('v') => {
                self.confirmation = Some(Confirmation::Action(Action::all(ActionKind::Validate)))
            }
            KeyCode::Char('+') => {
                self.confirmation = Some(Confirmation::Action(Action::all(ActionKind::Add)))
            }
            KeyCode::Backspace | KeyCode::Delete => self.confirm_selected(ActionKind::Delete),
            KeyCode::Char('u') => self.configure_selected(ActionKind::Update),
            KeyCode::Char('U') => self.configure_all(ActionKind::Update),
            KeyCode::Char('i') => self.configure_selected(ActionKind::AiUpdate),
            KeyCode::Char('I') => self.configure_all(ActionKind::AiUpdate),
            KeyCode::Char('x') => self.cancel_active()?,
            _ => {}
        }
        Ok(false)
    }

    fn handle_pending_keys(&mut self) -> Result<bool, VesshelmError> {
        let mut navigation_events = 0;
        loop {
            if let Event::Key(key) = event::read()?
                && key.kind == KeyEventKind::Press
            {
                if should_handle_navigation_event(key.code, &mut navigation_events)
                    && self.handle_key(key)?
                {
                    return Ok(true);
                }

                if self.interactive_action.is_some() {
                    return Ok(false);
                }
            }

            if !event::poll(Duration::ZERO)? {
                return Ok(false);
            }
        }
    }

    fn handle_search_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Enter => self.search_editing = false,
            KeyCode::Esc => {
                self.search_editing = false;
                self.clear_search();
            }
            KeyCode::Backspace => {
                self.search_query.pop();
                self.reset_visible_selection();
            }
            KeyCode::Char(character) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.search_query.push(character);
                self.reset_visible_selection();
            }
            _ => {}
        }
    }

    fn clear_search(&mut self) {
        self.search_query.clear();
        self.reset_visible_selection();
    }

    fn edit_selected_version(&mut self) {
        if self.child.is_some() {
            self.log("Wait for the running action to finish before changing a version.");
            return;
        }
        let Some(chart) = self.selected_chart() else {
            self.log("No chart available to edit.");
            return;
        };
        let Some(version) = &chart.version else {
            self.log(format!(
                "{} has no configured version and cannot be edited here.",
                chart.selection_label()
            ));
            return;
        };

        self.version_editor = Some(VersionEditor {
            chart_name: chart.name.clone(),
            namespace: chart.namespace.clone(),
            version: version.clone(),
        });
    }

    fn save_chart_version(&mut self, editor: &VersionEditor) {
        let version = editor.version.trim();
        if version.is_empty() || version.chars().any(char::is_whitespace) {
            self.log("Version must be a non-empty value without whitespace.");
            return;
        }

        match ConfigUpdater::update_chart_version_for_namespace(
            &self.config_path,
            &editor.chart_name,
            &editor.namespace,
            version,
        ) {
            Ok(()) => {
                self.log(format!(
                    "✓ Updated {} ({}) to {}.",
                    editor.chart_name, editor.namespace, version
                ));
                self.reload_config();
            }
            Err(error) => self.log(format!(
                "Could not update {} ({}): {error}",
                editor.chart_name, editor.namespace
            )),
        }
    }

    fn confirm_selected(&mut self, kind: ActionKind) {
        let charts = self.action_targets();
        if charts.is_empty() {
            self.log("Select a chart before starting an action.");
            return;
        }
        self.confirmation = Some(Confirmation::Action(Action::charts(kind, charts)));
    }

    fn configure_selected(&mut self, kind: ActionKind) {
        let charts = self.action_targets();
        if charts.is_empty() {
            self.log("Select a chart before starting an action.");
            return;
        }
        self.configure_action(Action::charts(kind, charts));
    }

    fn configure_all(&mut self, kind: ActionKind) {
        self.configure_action(Action::all(kind));
    }

    fn configure_action(&mut self, action: Action) {
        if action.options.fields().is_empty() {
            self.confirmation = Some(Confirmation::Action(action));
        } else {
            self.option_editor = Some(OptionEditor::new(action));
        }
    }

    fn start_action(&mut self, action: Action) -> Result<(), VesshelmError> {
        if self.child.is_some() {
            self.log("An action is already running. Cancel it before starting another one.");
            return Ok(());
        }

        if action.kind == ActionKind::Add {
            self.interactive_action = Some(action);
            return Ok(());
        }

        if action.kind == ActionKind::Delete
            && let ActionScope::Charts(charts) = &action.scope
            && charts.len() > 1
        {
            self.queued_actions = charts
                .iter()
                .cloned()
                .map(|chart| Action::charts(ActionKind::Delete, vec![chart]))
                .collect();
            return self.start_next_queued_action();
        }

        self.launch_action(action)
    }

    fn launch_action(&mut self, action: Action) -> Result<(), VesshelmError> {
        let executable = std::env::current_exe()?;
        let args = command_arguments(&self.config_path, &action);
        let mut command = Command::new(executable);
        command
            .args(args)
            // The child writes to pipes, so its terminal detection would
            // otherwise suppress the ANSI styles that we render in the TUI.
            .env("CLICOLOR_FORCE", "1")
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true);

        let mut child = command.spawn().map_err(|error| {
            VesshelmError::CommandFailed(format!("Could not start {}: {error}", action.label()))
        })?;
        let stdout = child.stdout.take().ok_or_else(|| {
            VesshelmError::CommandFailed("Could not capture action stdout".to_string())
        })?;
        let stderr = child.stderr.take().ok_or_else(|| {
            VesshelmError::CommandFailed("Could not capture action stderr".to_string())
        })?;
        let (output_tx, output_rx) = mpsc::unbounded_channel();
        tokio::spawn(forward_output(stdout, output_tx.clone()));
        tokio::spawn(forward_output(stderr, output_tx));

        self.clear_available_versions(&action);
        self.mark_action(&action, RunStatus::Running);
        self.activity.push_front(Activity {
            label: action.label(),
            status: RunStatus::Running,
        });
        self.activity.truncate(MAX_ACTIVITY);
        self.log(format!("→ Started {}", action.label()));
        self.start_pod_tracking(&action);
        self.child = Some(child);
        self.active_action = Some(action);
        self.output_rx = Some(output_rx);
        self.cancellation_requested = false;
        Ok(())
    }

    fn start_next_queued_action(&mut self) -> Result<(), VesshelmError> {
        let Some(action) = self.queued_actions.pop_front() else {
            return Ok(());
        };
        self.launch_action(action)
    }

    async fn run_interactive_action(
        &mut self,
        terminal: &mut ratatui::DefaultTerminal,
        action: Action,
    ) -> Result<(), VesshelmError> {
        self.activity.push_front(Activity {
            label: action.label(),
            status: RunStatus::Running,
        });
        self.activity.truncate(MAX_ACTIVITY);
        self.log("→ Opening the existing interactive add flow.");

        terminal.clear()?;
        ratatui::try_restore()?;

        let executable = std::env::current_exe()?;
        let status_result = Command::new(executable)
            .args(command_arguments(&self.config_path, &action))
            .status()
            .await;
        let restored_terminal = ratatui::try_init();
        let new_terminal = restored_terminal.map_err(VesshelmError::Io)?;
        *terminal = new_terminal;

        let status = status_result?;
        let final_status = if status.success() {
            RunStatus::Succeeded
        } else {
            RunStatus::Failed
        };
        if let Some(activity) = self.activity.front_mut() {
            activity.status = final_status;
        }
        self.log(format!(
            "← {} {} ({status})",
            action.label(),
            final_status.label()
        ));
        self.reload_config();
        Ok(())
    }

    fn cancel_active(&mut self) -> Result<(), VesshelmError> {
        let Some(child) = self.child.as_mut() else {
            self.log("No running action to cancel.");
            return Ok(());
        };
        if self.cancellation_requested {
            return Ok(());
        }

        child.start_kill()?;
        self.cancellation_requested = true;
        self.queued_actions.clear();
        self.log("→ Cancellation requested for the running action.");
        Ok(())
    }

    fn observe_child(&mut self) -> Result<(), VesshelmError> {
        let Some(child) = self.child.as_mut() else {
            return Ok(());
        };
        let Some(status) = child.try_wait()? else {
            return Ok(());
        };

        let final_status = if self.cancellation_requested {
            RunStatus::Canceled
        } else if status.success() {
            RunStatus::Succeeded
        } else {
            RunStatus::Failed
        };
        let should_reload_config = self.active_action.as_ref().is_some_and(|action| {
            final_status == RunStatus::Succeeded
                && matches!(
                    action.kind,
                    ActionKind::Sync
                        | ActionKind::Delete
                        | ActionKind::Update
                        | ActionKind::AiUpdate
                )
        });
        if let Some(action) = self.active_action.take() {
            self.mark_action(&action, final_status);
            if let Some(activity) = self.activity.front_mut() {
                activity.status = final_status;
            }
            self.log(format!(
                "← {} {} ({status})",
                action.label(),
                final_status.label()
            ));
        }
        self.child = None;
        self.cancellation_requested = false;
        if should_reload_config {
            self.reload_config();
        }
        self.start_next_queued_action()?;
        Ok(())
    }

    fn mark_action(&mut self, action: &Action, status: RunStatus) {
        match &action.scope {
            ActionScope::Charts(charts) => {
                for chart in charts {
                    self.chart_status.insert(chart.clone(), status);
                }
            }
            ActionScope::All if !matches!(action.kind, ActionKind::Validate | ActionKind::Add) => {
                for chart in &self.charts {
                    self.chart_status.insert(chart.name.clone(), status);
                }
            }
            ActionScope::All => {}
        }
    }

    fn drain_output(&mut self) {
        let mut lines = Vec::new();
        if let Some(output_rx) = self.output_rx.as_mut() {
            while let Ok(line) = output_rx.try_recv() {
                lines.push(line);
            }
        }
        for line in lines {
            if self
                .active_action
                .as_ref()
                .is_some_and(|action| action.kind == ActionKind::Update)
                && let Some((chart, latest)) = available_version(&line)
            {
                self.available_versions.insert(chart, latest);
            }
            self.log(line);
        }
    }

    fn clear_available_versions(&mut self, action: &Action) {
        if action.kind != ActionKind::Update {
            return;
        }
        match &action.scope {
            ActionScope::Charts(charts) => {
                for chart in charts {
                    self.available_versions.remove(chart);
                }
            }
            ActionScope::All => self.available_versions.clear(),
        }
    }

    fn start_pod_tracking(&mut self, action: &Action) {
        if action.kind != ActionKind::Deploy {
            return;
        }
        self.pod_targets = self
            .charts
            .iter()
            .filter(|chart| match &action.scope {
                ActionScope::Charts(names) => names.contains(&chart.name) && !chart.no_deploy,
                ActionScope::All => !chart.no_deploy,
            })
            .map(pod_target)
            .collect();
        self.pods.clear();
        self.pod_error = None;
        self.pod_refresh_in_flight = false;
        self.next_pod_refresh = Instant::now();
        if !self.pod_targets.is_empty() {
            let (pod_tx, pod_rx) = mpsc::unbounded_channel();
            self.pod_tx = Some(pod_tx);
            self.pod_rx = Some(pod_rx);
        }
    }

    fn refresh_pods(&mut self) {
        if self.pod_targets.is_empty()
            || self.pod_refresh_in_flight
            || Instant::now() < self.next_pod_refresh
        {
            return;
        }
        let Some(pod_tx) = self.pod_tx.as_ref() else {
            return;
        };
        let targets = self.pod_targets.clone();
        let pod_tx = pod_tx.clone();
        self.pod_refresh_in_flight = true;
        self.next_pod_refresh = Instant::now() + POD_REFRESH_INTERVAL;
        tokio::spawn(async move {
            let _ = pod_tx.send(fetch_pods(targets).await);
        });
    }

    fn drain_pod_updates(&mut self) {
        let mut updates = Vec::new();
        if let Some(pod_rx) = self.pod_rx.as_mut() {
            while let Ok(update) = pod_rx.try_recv() {
                updates.push(update);
            }
        }
        for update in updates {
            self.pod_targets = update.targets;
            self.pods = update.pods;
            self.pod_error = update.error;
            self.pod_refresh_in_flight = false;
        }
    }

    fn selected_chart(&self) -> Option<&Chart> {
        let visible = self.visible_chart_indices();
        let index = self.chart_state.selected()?;
        visible.get(index).and_then(|&index| self.charts.get(index))
    }

    fn visible_chart_indices(&self) -> Vec<usize> {
        let query = self.search_query.to_lowercase();
        self.charts
            .iter()
            .enumerate()
            .filter(|(_, chart)| {
                query.is_empty()
                    || chart.name.to_lowercase().contains(&query)
                    || chart.namespace.to_lowercase().contains(&query)
                    || chart
                        .repo_name
                        .as_deref()
                        .is_some_and(|repo| repo.to_lowercase().contains(&query))
                    || chart
                        .version
                        .as_deref()
                        .is_some_and(|version| version.to_lowercase().contains(&query))
            })
            .map(|(index, _)| index)
            .collect()
    }

    fn reset_visible_selection(&mut self) {
        self.chart_state
            .select((!self.visible_chart_indices().is_empty()).then_some(0));
    }

    fn action_targets(&self) -> Vec<String> {
        if self.selected_charts.is_empty() {
            return self
                .selected_chart()
                .map(|chart| vec![chart.name.clone()])
                .unwrap_or_default();
        }

        self.charts
            .iter()
            .filter(|chart| self.selected_charts.contains(&chart.name))
            .map(|chart| chart.name.clone())
            .collect()
    }

    fn toggle_selected_chart(&mut self) {
        let Some(chart) = self.selected_chart() else {
            self.log("No chart available to select.");
            return;
        };
        let name = chart.name.clone();
        let selected = if self.selected_charts.insert(name.clone()) {
            true
        } else {
            self.selected_charts.remove(&name);
            false
        };
        let verb = if selected { "Selected" } else { "Deselected" };
        self.log(format!("{verb} {name}."));
    }

    fn toggle_all_charts(&mut self) {
        if self.charts.is_empty() {
            self.log("No chart available to select.");
            return;
        }

        if self.selected_charts.len() == self.charts.len() {
            self.selected_charts.clear();
            self.log("Cleared chart selection.");
        } else {
            self.selected_charts = self.charts.iter().map(|chart| chart.name.clone()).collect();
            self.log(format!("Selected all {} chart(s).", self.charts.len()));
        }
    }

    fn select_next(&mut self) {
        let visible = self.visible_chart_indices();
        if visible.is_empty() {
            self.chart_state.select(None);
            return;
        }
        let index = self.chart_state.selected().unwrap_or(0);
        self.chart_state.select(Some((index + 1) % visible.len()));
    }

    fn select_previous(&mut self) {
        let visible = self.visible_chart_indices();
        if visible.is_empty() {
            self.chart_state.select(None);
            return;
        }
        let index = self.chart_state.selected().unwrap_or(0);
        let previous = if index == 0 {
            visible.len() - 1
        } else {
            index - 1
        };
        self.chart_state.select(Some(previous));
    }

    fn log(&mut self, line: impl Into<String>) {
        self.logs.push_back(parse_ansi_log_line(&line.into()));
        if self.logs.len() > MAX_LOG_LINES {
            self.logs.pop_front();
        }
    }

    fn draw(&mut self, frame: &mut Frame) {
        let area = frame.area();
        if area.width < 50 || area.height < 12 {
            frame.render_widget(
                Paragraph::new("Terminal too small — resize to at least 50×12.")
                    .alignment(Alignment::Center)
                    .block(Block::bordered().title("Vesshelm")),
                area,
            );
            return;
        }

        if self.fullscreen_logs {
            self.draw_fullscreen_logs(frame, area);
            if self.show_help {
                self.draw_help(frame, area);
            }
            return;
        }

        let [header, content, footer] = Layout::vertical([
            Constraint::Length(3),
            Constraint::Min(7),
            Constraint::Length(2),
        ])
        .areas(area);
        let [overview, logs] =
            Layout::vertical([Constraint::Length(13), Constraint::Min(4)]).areas(content);
        let [charts, sidebar] =
            Layout::horizontal([Constraint::Percentage(60), Constraint::Percentage(40)])
                .areas(overview);
        let [activity, pods] =
            Layout::vertical([Constraint::Length(5), Constraint::Min(7)]).areas(sidebar);

        self.draw_header(frame, header);
        self.draw_charts(frame, charts);
        self.draw_activity(frame, activity);
        self.draw_pods(frame, pods);
        self.draw_logs(frame, logs);
        self.draw_footer(frame, footer);

        if self.show_help {
            self.draw_help(frame, area);
        }
        if let Some(confirmation) = &self.confirmation {
            draw_confirmation(frame, area, &confirmation.message());
        }
        if let Some(editor) = &self.option_editor {
            draw_option_editor(frame, area, editor);
        }
        if let Some(editor) = &self.version_editor {
            draw_version_editor(frame, area, editor);
        }
    }

    fn draw_header(&self, frame: &mut Frame, area: Rect) {
        let mut state = self.active_action.as_ref().map_or_else(
            || "READY · select a chart then launch an action".to_string(),
            |action| format!("RUNNING · {}", action.label()),
        );
        if !self.search_query.is_empty() {
            state.push_str(&format!(
                " · FILTER `{}` ({}/{})",
                self.search_query,
                self.visible_chart_indices().len(),
                self.charts.len()
            ));
        }
        let subtitle = self.config_error.as_deref().unwrap_or(&state);
        let text = Text::from(vec![
            Line::from(vec![
                Span::styled(
                    " VESSHELM ",
                    Style::new()
                        .fg(Color::Black)
                        .bg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    "  control plane",
                    Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(Span::styled(
                format!(" {subtitle}"),
                Style::new().fg(Color::Gray),
            )),
        ]);
        frame.render_widget(
            Paragraph::new(text).block(
                Block::new()
                    .borders(Borders::BOTTOM)
                    .border_style(Style::new().fg(Color::DarkGray)),
            ),
            area,
        );
    }

    fn draw_charts(&mut self, frame: &mut Frame, area: Rect) {
        let header = Row::new([
            "MARK",
            "STATUS",
            "CHART",
            "NAMESPACE",
            "VERSION",
            "AVAILABLE",
            "SYNC",
            "FLAGS",
        ])
        .style(Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD));
        let visible = self.visible_chart_indices();
        let rows = visible.iter().map(|&index| {
            let chart = &self.charts[index];
            let status = self
                .chart_status
                .get(&chart.name)
                .copied()
                .unwrap_or(RunStatus::Idle);
            let mut flags = Vec::new();
            if chart.no_sync {
                flags.push("no-sync");
            }
            if chart.no_deploy {
                flags.push("no-deploy");
            }
            let sync_state = sync_state(chart, &self.synced_versions);
            let marked = self.selected_charts.contains(&chart.name);
            Row::new(vec![
                Cell::from(if marked { "[x]" } else { "[ ]" }).style(Style::new().fg(if marked {
                    Color::Green
                } else {
                    Color::DarkGray
                })),
                Cell::from(status.label())
                    .style(Style::new().fg(status.color()).add_modifier(Modifier::BOLD)),
                Cell::from(chart.name.clone()),
                Cell::from(chart.namespace.clone()),
                Cell::from(chart.version.clone().unwrap_or_else(|| "local".to_string())),
                Cell::from(
                    self.available_versions
                        .get(&chart.name)
                        .cloned()
                        .unwrap_or_else(|| "—".to_string()),
                )
                .style(if self.available_versions.contains_key(&chart.name) {
                    Style::new().fg(Color::Green).add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                }),
                Cell::from(sync_state.label().to_string()).style(sync_state.style()),
                Cell::from(flags.join(", ")),
            ])
        });
        let title = format!(
            " Charts ({}/{} · {} selected) ",
            visible.len(),
            self.charts.len(),
            self.selected_charts.len()
        );
        let table = Table::new(
            rows,
            [
                Constraint::Length(6),
                Constraint::Length(10),
                Constraint::Percentage(24),
                Constraint::Percentage(15),
                Constraint::Percentage(14),
                Constraint::Percentage(14),
                Constraint::Percentage(18),
                Constraint::Percentage(15),
            ],
        )
        .header(header)
        .block(
            Block::bordered()
                .title(title)
                .border_style(Style::new().fg(Color::Blue)),
        )
        .row_highlight_style(
            Style::new()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("› ");
        frame.render_stateful_widget(table, area, &mut self.chart_state);
    }

    fn draw_activity(&self, frame: &mut Frame, area: Rect) {
        let items = if self.activity.is_empty() {
            vec![ListItem::new(Span::styled(
                "No action yet",
                Style::new().fg(Color::DarkGray),
            ))]
        } else {
            self.activity
                .iter()
                .map(|entry| {
                    ListItem::new(Line::from(vec![
                        Span::styled(
                            format!("{:>9} ", entry.status.label()),
                            Style::new()
                                .fg(entry.status.color())
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::raw(&entry.label),
                    ]))
                })
                .collect()
        };
        frame.render_widget(
            List::new(items).block(
                Block::bordered()
                    .title(" Activity ")
                    .border_style(Style::new().fg(Color::Blue)),
            ),
            area,
        );
    }

    fn draw_pods(&self, frame: &mut Frame, area: Rect) {
        let title = if self.pod_targets.is_empty() {
            " Pods (last deployment) "
        } else {
            " Pods (live · 1s) "
        };
        if let Some(error) = &self.pod_error {
            frame.render_widget(
                Paragraph::new(error.as_str())
                    .wrap(Wrap { trim: true })
                    .block(
                        Block::bordered()
                            .title(title)
                            .border_style(Style::new().fg(Color::Red)),
                    ),
                area,
            );
            return;
        }

        let rows = self.pods.iter().map(|pod| {
            let phase_color = match pod.phase.as_str() {
                "Running" => Color::Green,
                "Succeeded" => Color::Cyan,
                "Pending" => Color::Yellow,
                _ => Color::Red,
            };
            Row::new(vec![
                Cell::from(pod.chart.clone()),
                Cell::from(pod.name.clone()),
                Cell::from(pod.age.clone()),
                Cell::from(pod.phase.clone()).style(Style::new().fg(phase_color)),
                Cell::from(pod.ready.clone()),
                Cell::from(pod.restarts.to_string()),
            ])
        });
        let table = Table::new(
            rows,
            [
                Constraint::Percentage(17),
                Constraint::Percentage(31),
                Constraint::Percentage(11),
                Constraint::Percentage(16),
                Constraint::Percentage(10),
                Constraint::Percentage(15),
            ],
        )
        .header(
            Row::new(["CHART", "POD", "AGE", "PHASE", "READY", "RESTARTS"])
                .style(Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        )
        .block(
            Block::bordered()
                .title(title)
                .border_style(Style::new().fg(Color::Blue)),
        );
        frame.render_widget(table, area);
    }

    fn draw_logs(&self, frame: &mut Frame, area: Rect) {
        let inner_height = area.height.saturating_sub(2) as usize;
        let max_scroll = self.logs.len().saturating_sub(inner_height) as u16;
        let scroll = max_scroll.saturating_sub(self.log_scroll);
        let lines: Vec<Line<'static>> = self.logs.iter().map(LogLine::to_line).collect();
        let title = if self.log_scroll == 0 {
            " Live logs · following · l full screen "
        } else {
            " Live logs · paused (g to follow) · l full screen "
        };
        let logs = Paragraph::new(lines)
            .block(
                Block::bordered()
                    .title(title)
                    .border_style(Style::new().fg(Color::Magenta)),
            )
            .scroll((scroll, 0))
            .wrap(Wrap { trim: false });
        frame.render_widget(logs, area);
    }

    fn draw_fullscreen_logs(&self, frame: &mut Frame, area: Rect) {
        let [header, content, footer] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Min(1),
            Constraint::Length(1),
        ])
        .areas(area);
        let inner_height = content.height as usize;
        let max_scroll = self.logs.len().saturating_sub(inner_height) as u16;
        let scroll = max_scroll.saturating_sub(self.log_scroll);
        let state = if self.log_scroll == 0 {
            "following"
        } else {
            "paused"
        };
        frame.render_widget(
            Paragraph::new(Line::from(vec![
                Span::styled(
                    " LIVE LOGS ",
                    Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                ),
                Span::styled(format!("{state}  "), Style::new().fg(Color::Gray)),
                Span::styled("plain text copy", Style::new().fg(Color::DarkGray)),
            ])),
            header,
        );
        frame.render_widget(
            Paragraph::new(self.logs.iter().map(LogLine::to_line).collect::<Vec<_>>())
                .scroll((scroll, 0))
                .wrap(Wrap { trim: false }),
            content,
        );
        frame.render_widget(
            Paragraph::new(Line::from(vec![
                Span::styled(
                    " Esc / l ",
                    Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                ),
                Span::styled("dashboard  ", Style::new().fg(Color::Gray)),
                Span::styled(
                    "PgUp/PgDn",
                    Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                ),
                Span::styled(" scroll  ", Style::new().fg(Color::Gray)),
                Span::styled(
                    "g",
                    Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                ),
                Span::styled(" follow", Style::new().fg(Color::Gray)),
            ]))
            .alignment(Alignment::Center),
            footer,
        );
    }

    fn draw_footer(&self, frame: &mut Frame, area: Rect) {
        let text = if self.search_editing {
            Line::from(vec![
                Span::styled(
                    " / ",
                    Style::new()
                        .fg(Color::Black)
                        .bg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!("{}▏", self.search_query),
                    Style::new().fg(Color::Cyan),
                ),
                Span::styled(
                    "  Enter apply · Esc clear",
                    Style::new().fg(Color::DarkGray),
                ),
            ])
        } else if !self.search_query.is_empty() {
            Line::from(vec![
                Span::styled(
                    " / ",
                    Style::new()
                        .fg(Color::Black)
                        .bg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(&self.search_query, Style::new().fg(Color::Cyan)),
                Span::styled(
                    "  filtered · / edit · Esc clear",
                    Style::new().fg(Color::DarkGray),
                ),
            ])
        } else {
            let mut spans = Vec::new();
            if self.selected_charts.is_empty() {
                spans.extend([
                    Span::styled(
                        " ↑↓ ",
                        Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                    ),
                    Span::styled("move  ", Style::new().fg(Color::Gray)),
                    Span::styled(
                        " Space ",
                        Style::new().fg(Color::Green).add_modifier(Modifier::BOLD),
                    ),
                    Span::styled("mark  ", Style::new().fg(Color::Gray)),
                    Span::styled(
                        " d/D ",
                        Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                    ),
                    Span::styled("deploy  ", Style::new().fg(Color::Gray)),
                    Span::styled(
                        " u/U ",
                        Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                    ),
                    Span::styled("update  ", Style::new().fg(Color::Gray)),
                    Span::styled(
                        " / ",
                        Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                    ),
                    Span::styled("filter  ", Style::new().fg(Color::Gray)),
                    Span::styled(
                        " ? ",
                        Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                    ),
                    Span::styled("help  ", Style::new().fg(Color::Gray)),
                    Span::styled(
                        " q ",
                        Style::new()
                            .fg(Color::DarkGray)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled("quit", Style::new().fg(Color::DarkGray)),
                ]);
            } else {
                spans.extend([
                    Span::styled(
                        format!(" {} marked  ", self.selected_charts.len()),
                        Style::new().fg(Color::Green).add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        "d/D",
                        Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(" deploy  ", Style::new().fg(Color::Gray)),
                    Span::styled(
                        "u/U",
                        Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(" update  ", Style::new().fg(Color::Gray)),
                    Span::styled(
                        "i/I",
                        Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(" AI  ", Style::new().fg(Color::Gray)),
                    Span::styled(
                        "s/S",
                        Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(" sync  ", Style::new().fg(Color::Gray)),
                    Span::styled(
                        "/",
                        Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(" filter  ", Style::new().fg(Color::Gray)),
                    Span::styled(
                        "?",
                        Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(" help  ", Style::new().fg(Color::Gray)),
                    Span::styled(
                        "q",
                        Style::new()
                            .fg(Color::DarkGray)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(" quit", Style::new().fg(Color::DarkGray)),
                ]);
            }
            Line::from(spans)
        };
        frame.render_widget(Paragraph::new(text).alignment(Alignment::Center), area);
    }

    fn draw_help(&self, frame: &mut Frame, area: Rect) {
        let popup = centered_rect(96, 78, area);
        frame.render_widget(Clear, popup);
        let block = Block::bordered()
            .title(" Help · keyboard map ")
            .border_style(Style::new().fg(Color::Cyan));
        let inner = block.inner(popup);
        frame.render_widget(block, popup);

        let [content, footer] =
            Layout::vertical([Constraint::Min(1), Constraint::Length(1)]).areas(inner);
        let [left, right] =
            Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
                .areas(content);
        let [left_padding, left_content] =
            Layout::horizontal([Constraint::Length(1), Constraint::Min(1)]).areas(left);
        let [right_content, right_padding] =
            Layout::horizontal([Constraint::Min(1), Constraint::Length(1)]).areas(right);
        let _ = (left_padding, right_padding);

        let left_lines = vec![
            help_section("NAVIGATE & SELECT"),
            help_shortcut("↑ ↓ / j k", "move cursor"),
            help_shortcut("Space", "mark / unmark chart"),
            help_shortcut("a", "mark all / clear"),
            Line::default(),
            help_section("RUN"),
            help_shortcut("s / d", "configure sync / deploy selected"),
            help_shortcut("S / D", "configure sync / deploy all"),
            help_shortcut("x", "cancel action"),
            help_shortcut("l", "full-screen logs / copy view"),
        ];
        let right_lines = vec![
            help_section("LIFECYCLE"),
            help_shortcut("+", "open add assistant"),
            help_shortcut("e", "edit focused chart version"),
            help_shortcut("Backspace", "delete selected charts"),
            help_shortcut("u / U", "configure update / all"),
            help_shortcut("i / I", "configure AI update / all"),
            Line::default(),
            help_section("SYSTEM"),
            help_shortcut("v", "validate configuration"),
            help_shortcut("r", "reload configuration"),
            help_shortcut("PgUp / PgDn", "browse logs"),
            help_shortcut("g", "resume live log tail"),
            help_shortcut("q / Ctrl-C", "quit dashboard"),
        ];
        frame.render_widget(
            Paragraph::new(left_lines).wrap(Wrap { trim: false }),
            left_content,
        );
        frame.render_widget(
            Paragraph::new(right_lines).wrap(Wrap { trim: false }),
            right_content,
        );
        frame.render_widget(
            Paragraph::new(Line::from(vec![
                Span::styled(
                    "Esc  ?  h",
                    Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                ),
                Span::styled("  close help", Style::new().fg(Color::DarkGray)),
            ]))
            .alignment(Alignment::Center),
            footer,
        );
    }
}

fn help_section(title: &str) -> Line<'static> {
    Line::from(Span::styled(
        title.to_string(),
        Style::new().fg(Color::Blue).add_modifier(Modifier::BOLD),
    ))
}

fn help_shortcut(keys: &str, description: &str) -> Line<'static> {
    Line::from(vec![
        Span::styled(
            format!("{keys:>12}"),
            Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
        Span::styled(description.to_string(), Style::new().fg(Color::Gray)),
    ])
}

fn command_arguments(config_path: &Path, action: &Action) -> Vec<OsString> {
    let mut args = vec![
        OsString::from("--config"),
        config_path.as_os_str().to_os_string(),
        OsString::from(action.kind.label()),
    ];
    // `update` uses this flag to suppress its per-chart results and summary.
    // Those lines are exactly what the dashboard needs to surface in live logs.
    // Other actions can emit persistent terminal progress widgets, which are not
    // suitable for a line-oriented log pane.
    if action.kind != ActionKind::Update {
        args.push(OsString::from("--no-progress"));
    }
    match action.kind {
        ActionKind::Sync => {
            if let ActionScope::Charts(charts) = &action.scope {
                args.extend(charts.iter().map(OsString::from));
            }
            if let ActionOptions::Sync { ignore_skip, jobs } = &action.options {
                if *ignore_skip {
                    args.push(OsString::from("--ignore-skip"));
                }
                if *jobs != 5 {
                    args.extend([OsString::from("--jobs"), OsString::from(jobs.to_string())]);
                }
            }
        }
        ActionKind::Deploy => {
            match &action.scope {
                ActionScope::Charts(charts) => args.extend(charts.iter().map(OsString::from)),
                ActionScope::All => args.push(OsString::from("all")),
            }
            // The TUI has already obtained an explicit confirmation. The child
            // must never compete with the dashboard for stdin.
            args.push(OsString::from("--no-interactive"));
            if let ActionOptions::Deploy {
                dry_run,
                take_ownership,
                force,
                no_diff,
                debug,
            } = &action.options
            {
                if *dry_run {
                    args.push(OsString::from("--dry-run"));
                }
                if *take_ownership {
                    args.push(OsString::from("--take-ownership"));
                }
                if *force {
                    args.push(OsString::from("--force"));
                }
                if *no_diff {
                    args.push(OsString::from("--no-diff"));
                }
                if *debug {
                    args.push(OsString::from("--debug"));
                }
            }
        }
        ActionKind::Update => {
            if let ActionScope::Charts(charts) = &action.scope {
                args.extend(charts.iter().map(OsString::from));
            }
            if let ActionOptions::Update { mode, jobs } = &action.options {
                match mode {
                    UpdateMode::Check => {}
                    UpdateMode::Apply => args.push(OsString::from("--apply")),
                    UpdateMode::ApplySync => args.push(OsString::from("--apply-sync")),
                }
                if *jobs != 5 {
                    args.extend([OsString::from("--jobs"), OsString::from(jobs.to_string())]);
                }
            }
        }
        ActionKind::AiUpdate => {
            if let ActionScope::Charts(charts) = &action.scope {
                args.extend(charts.iter().map(OsString::from));
            }
            if let ActionOptions::AiUpdate {
                re_analyze,
                jobs,
                debug_ai,
            } = &action.options
            {
                if *re_analyze {
                    args.push(OsString::from("--re-analyze"));
                }
                if *jobs != 5 {
                    args.extend([OsString::from("--jobs"), OsString::from(jobs.to_string())]);
                }
                if *debug_ai {
                    args.push(OsString::from("--debug-ai"));
                }
            }
            args.push(OsString::from("--no-interactive"));
        }
        ActionKind::Delete => {
            if let ActionScope::Charts(charts) = &action.scope
                && let Some(chart) = charts.first()
            {
                args.push(OsString::from(chart));
                args.push(OsString::from("--no-interactive"));
            }
        }
        ActionKind::Validate | ActionKind::Add => {}
    }
    args
}

fn available_version(line: &str) -> Option<(String, String)> {
    let line = console::strip_ansi_codes(line);
    let (chart, update) = line
        .strip_prefix("checking ")?
        .split_once("... Outdated ")?;
    let (_, latest) = update.split_once(" -> ")?;
    Some((
        chart.to_string(),
        latest.split_whitespace().next()?.to_string(),
    ))
}

fn sync_state(
    chart: &Chart,
    synced_versions: &HashMap<String, HashMap<String, String>>,
) -> SyncState {
    let (Some(repo_name), Some(version)) = (&chart.repo_name, &chart.version) else {
        return SyncState::NotApplicable;
    };
    if chart.no_sync {
        return SyncState::NotApplicable;
    }
    match synced_versions
        .get(&chart.name)
        .and_then(|versions| versions.get(repo_name))
    {
        Some(synced) if synced == version => SyncState::Synced,
        Some(synced) => SyncState::Pending(format!("{synced} → {version}")),
        None => SyncState::Pending("not synced".to_string()),
    }
}

fn should_handle_navigation_event(key: KeyCode, navigation_events: &mut usize) -> bool {
    if matches!(
        key,
        KeyCode::Down | KeyCode::Up | KeyCode::Char('j') | KeyCode::Char('k')
    ) {
        let should_handle = *navigation_events < MAX_NAVIGATION_EVENTS_PER_FRAME;
        *navigation_events += 1;
        should_handle
    } else {
        *navigation_events = 0;
        true
    }
}

fn pod_target(chart: &Chart) -> PodTarget {
    PodTarget {
        chart: chart.name.clone(),
        namespace: chart.namespace.clone(),
        release: chart.release_name(),
        selectors: release_selectors(&chart.release_name()),
        selectors_resolved: false,
    }
}

fn release_selectors(release: &str) -> Vec<HashMap<String, String>> {
    ["app.kubernetes.io/instance", "release"]
        .into_iter()
        .map(|label| HashMap::from([(label.to_string(), release.to_string())]))
        .collect()
}

async fn fetch_pods(mut targets: Vec<PodTarget>) -> PodUpdate {
    let mut pods = HashMap::new();
    let mut last_error = None;
    let mut succeeded = false;
    for target in &mut targets {
        if !target.selectors_resolved
            && let Err(error) = resolve_pod_selectors(target).await
        {
            last_error = Some(error);
        }
    }
    let mut targets_by_namespace: HashMap<String, Vec<PodTarget>> = HashMap::new();
    for target in targets.clone() {
        targets_by_namespace
            .entry(target.namespace.clone())
            .or_default()
            .push(target);
    }

    for (namespace, targets) in targets_by_namespace {
        match Command::new("kubectl")
            .args([
                "get",
                "pods",
                "--namespace",
                namespace.as_str(),
                "--output",
                "json",
            ])
            .output()
            .await
        {
            Ok(output) if output.status.success() => {
                succeeded = true;
                match parse_pods(&targets, &output.stdout) {
                    Ok(found) => {
                        for pod in found {
                            pods.insert(format!("{namespace}/{}", pod.name), pod);
                        }
                    }
                    Err(error) => last_error = Some(error),
                }
            }
            Ok(output) => {
                last_error = Some(String::from_utf8_lossy(&output.stderr).trim().to_string());
            }
            Err(error) => last_error = Some(format!("kubectl unavailable: {error}")),
        }
    }

    let mut pods: Vec<_> = pods.into_values().collect();
    pods.sort_by(|left, right| left.name.cmp(&right.name));
    PodUpdate {
        pods,
        error: (!succeeded)
            .then_some(last_error.unwrap_or_else(|| "Could not query Pods.".to_string())),
        targets,
    }
}

async fn resolve_pod_selectors(target: &mut PodTarget) -> Result<(), String> {
    let output = Command::new("helm")
        .args([
            "get",
            "manifest",
            target.release.as_str(),
            "--namespace",
            target.namespace.as_str(),
        ])
        .output()
        .await
        .map_err(|error| format!("helm unavailable: {error}"))?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).trim().to_string());
    }
    for selector in manifest_pod_selectors(&String::from_utf8_lossy(&output.stdout))? {
        if !target.selectors.contains(&selector) {
            target.selectors.push(selector);
        }
    }
    target.selectors_resolved = true;
    Ok(())
}

fn manifest_pod_selectors(manifest: &str) -> Result<Vec<HashMap<String, String>>, String> {
    let mut selectors = Vec::new();
    for document in serde_yaml_ng::Deserializer::from_str(manifest) {
        let value = serde_yaml_ng::Value::deserialize(document)
            .map_err(|error| format!("Could not decode Helm manifest: {error}"))?;
        let kind = value.get("kind").and_then(serde_yaml_ng::Value::as_str);
        let path = match kind {
            Some("Job") => Some(&["spec", "template", "metadata", "labels"][..]),
            Some("CronJob") => Some(
                &[
                    "spec",
                    "jobTemplate",
                    "spec",
                    "template",
                    "metadata",
                    "labels",
                ][..],
            ),
            Some("Pod") => Some(&["metadata", "labels"][..]),
            Some("Deployment" | "StatefulSet" | "DaemonSet" | "ReplicaSet") => {
                Some(&["spec", "selector", "matchLabels"][..])
            }
            _ => None,
        };
        if let Some(path) = path
            && let Some(selector) = labels_at(&value, path)
            && !selectors.contains(&selector)
        {
            selectors.push(selector);
        }
    }
    Ok(selectors)
}

fn labels_at(value: &serde_yaml_ng::Value, path: &[&str]) -> Option<HashMap<String, String>> {
    let labels = path
        .iter()
        .try_fold(value, |value, key| value.get(*key))?
        .as_mapping()?;
    let labels: HashMap<_, _> = labels
        .iter()
        .filter_map(|(key, value)| Some((key.as_str()?.to_string(), value.as_str()?.to_string())))
        .collect();
    (!labels.is_empty()).then_some(labels)
}

fn parse_pods(targets: &[PodTarget], output: &[u8]) -> Result<Vec<TrackedPod>, String> {
    let list: KubernetesPodList = serde_json::from_slice(output)
        .map_err(|error| format!("Could not decode kubectl Pods response: {error}"))?;
    Ok(list
        .items
        .into_iter()
        .filter_map(|pod| {
            let chart = targets
                .iter()
                .find(|target| {
                    target.selectors.iter().any(|selector| {
                        selector.iter().all(|(label, value)| {
                            pod.metadata
                                .labels
                                .get(label)
                                .is_some_and(|actual| actual == value)
                        })
                    })
                })
                .map(|target| target.chart.as_str())?;
            if pod.metadata.name.is_empty() {
                return None;
            }
            let total = pod.status.container_statuses.len();
            let ready = pod
                .status
                .container_statuses
                .iter()
                .filter(|status| status.ready)
                .count();
            let restarts = pod
                .status
                .container_statuses
                .iter()
                .map(|status| status.restart_count)
                .sum();
            Some(TrackedPod {
                chart: chart.to_string(),
                name: pod.metadata.name,
                age: pod_age(pod.metadata.creation_timestamp.as_deref()),
                phase: pod.status.phase.unwrap_or_else(|| "Unknown".to_string()),
                ready: format!("{ready}/{total}"),
                restarts,
            })
        })
        .collect())
}

fn pod_age(timestamp: Option<&str>) -> String {
    timestamp
        .and_then(kubernetes_timestamp)
        .and_then(|created| SystemTime::now().duration_since(created).ok())
        .map(format_age)
        .unwrap_or_else(|| "—".to_string())
}

fn kubernetes_timestamp(timestamp: &str) -> Option<SystemTime> {
    // ponytail: Kubernetes serializes creationTimestamp as UTC RFC3339; use a parser crate if offsets matter.
    let year = timestamp.get(0..4)?.parse::<i64>().ok()?;
    let month = timestamp.get(5..7)?.parse::<i64>().ok()?;
    let day = timestamp.get(8..10)?.parse::<i64>().ok()?;
    let hour = timestamp.get(11..13)?.parse::<u64>().ok()?;
    let minute = timestamp.get(14..16)?.parse::<u64>().ok()?;
    let second = timestamp.get(17..19)?.parse::<u64>().ok()?;
    let days = days_from_civil(year, month, day);
    let seconds = days
        .checked_mul(86_400)?
        .checked_add((hour * 3_600 + minute * 60 + second) as i64)?;
    (seconds >= 0).then_some(UNIX_EPOCH + Duration::from_secs(seconds as u64))
}

fn days_from_civil(year: i64, month: i64, day: i64) -> i64 {
    let year = year - i64::from(month <= 2);
    let era = (if year >= 0 { year } else { year - 399 }) / 400;
    let year_of_era = year - era * 400;
    let day_of_year = (153 * (month + if month > 2 { -3 } else { 9 }) + 2) / 5 + day - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    era * 146_097 + day_of_era - 719_468
}

fn format_age(age: Duration) -> String {
    match age.as_secs() {
        seconds if seconds >= 86_400 => format!("{}d", seconds / 86_400),
        seconds if seconds >= 3_600 => format!("{}h", seconds / 3_600),
        seconds if seconds >= 60 => format!("{}m", seconds / 60),
        seconds => format!("{seconds}s"),
    }
}

fn parse_ansi_log_line(line: &str) -> LogLine {
    let mut spans = Vec::new();
    let mut style = Style::new();
    let mut text_start = 0;
    let mut cursor = 0;

    while let Some(escape_offset) = line[cursor..].find('\u{1b}') {
        let escape_start = cursor + escape_offset;
        if !line[escape_start..].starts_with("\u{1b}[") {
            cursor = escape_start + '\u{1b}'.len_utf8();
            continue;
        }

        let sequence_start = escape_start + 2;
        let Some(sequence_end) = line[sequence_start..]
            .bytes()
            .position(|byte| byte.is_ascii_alphabetic())
            .map(|offset| sequence_start + offset)
        else {
            break;
        };

        push_log_span(&mut spans, &line[text_start..escape_start], style);
        if line.as_bytes()[sequence_end] == b'm' {
            apply_sgr(&mut style, &line[sequence_start..sequence_end]);
        }
        cursor = sequence_end + 1;
        text_start = cursor;
    }
    push_log_span(&mut spans, &line[text_start..], style);

    LogLine { spans }
}

fn push_log_span(spans: &mut Vec<LogSpan>, content: &str, style: Style) {
    if !content.is_empty() {
        spans.push(LogSpan {
            content: content.to_string(),
            style,
        });
    }
}

fn apply_sgr(style: &mut Style, params: &str) {
    let params: Vec<u16> = if params.is_empty() {
        vec![0]
    } else {
        params
            .split(';')
            .filter_map(|param| param.parse::<u16>().ok())
            .collect()
    };
    let mut index = 0;
    while let Some(&param) = params.get(index) {
        match param {
            0 => *style = Style::new(),
            1 => *style = (*style).add_modifier(Modifier::BOLD),
            2 => *style = (*style).add_modifier(Modifier::DIM),
            3 => *style = (*style).add_modifier(Modifier::ITALIC),
            4 => *style = (*style).add_modifier(Modifier::UNDERLINED),
            7 => *style = (*style).add_modifier(Modifier::REVERSED),
            9 => *style = (*style).add_modifier(Modifier::CROSSED_OUT),
            22 => *style = (*style).remove_modifier(Modifier::BOLD | Modifier::DIM),
            23 => *style = (*style).remove_modifier(Modifier::ITALIC),
            24 => *style = (*style).remove_modifier(Modifier::UNDERLINED),
            27 => *style = (*style).remove_modifier(Modifier::REVERSED),
            29 => *style = (*style).remove_modifier(Modifier::CROSSED_OUT),
            30..=37 | 90..=97 => {
                if let Some(color) = ansi_color(param) {
                    *style = (*style).fg(color);
                }
            }
            39 => style.fg = None,
            40..=47 | 100..=107 => {
                if let Some(color) = ansi_color(param - 10) {
                    *style = (*style).bg(color);
                }
            }
            49 => style.bg = None,
            38 | 48 => {
                let foreground = param == 38;
                if let Some((color, consumed)) = extended_ansi_color(&params[index + 1..]) {
                    if foreground {
                        *style = (*style).fg(color);
                    } else {
                        *style = (*style).bg(color);
                    }
                    index += consumed;
                }
            }
            _ => {}
        }
        index += 1;
    }
}

fn extended_ansi_color(params: &[u16]) -> Option<(Color, usize)> {
    match params {
        [5, color, ..] if *color <= u8::MAX.into() => Some((Color::Indexed(*color as u8), 2)),
        [2, red, green, blue, ..]
            if *red <= u8::MAX.into() && *green <= u8::MAX.into() && *blue <= u8::MAX.into() =>
        {
            Some((Color::Rgb(*red as u8, *green as u8, *blue as u8), 4))
        }
        _ => None,
    }
}

fn ansi_color(code: u16) -> Option<Color> {
    Some(match code {
        30 => Color::Black,
        31 => Color::Red,
        32 => Color::Green,
        33 => Color::Yellow,
        34 => Color::Blue,
        35 => Color::Magenta,
        36 => Color::Cyan,
        37 => Color::Gray,
        90 => Color::DarkGray,
        91 => Color::LightRed,
        92 => Color::LightGreen,
        93 => Color::LightYellow,
        94 => Color::LightBlue,
        95 => Color::LightMagenta,
        96 => Color::LightCyan,
        97 => Color::White,
        _ => return None,
    })
}

async fn forward_output<R>(stream: R, output_tx: mpsc::UnboundedSender<String>)
where
    R: AsyncRead + Unpin,
{
    let mut lines = BufReader::new(stream).lines();
    while let Ok(Some(line)) = lines.next_line().await {
        if output_tx.send(line).is_err() {
            break;
        }
    }
}

fn draw_confirmation(frame: &mut Frame, area: Rect, message: &str) {
    let popup = centered_rect(55, 26, area);
    frame.render_widget(Clear, popup);
    let text = Text::from(vec![
        Line::from(Span::styled(
            message,
            Style::new().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from("Enter / y: confirm     Esc / n: cancel"),
    ]);
    frame.render_widget(
        Paragraph::new(text)
            .alignment(Alignment::Center)
            .block(
                Block::bordered()
                    .title(" Confirm action ")
                    .border_style(Style::new().fg(Color::Yellow)),
            )
            .wrap(Wrap { trim: false }),
        popup,
    );
}

fn draw_option_editor(frame: &mut Frame, area: Rect, editor: &OptionEditor) {
    let popup = centered_rect(64, 52, area);
    frame.render_widget(Clear, popup);
    let block = Block::bordered()
        .title(format!(" {} options ", editor.action.kind.label()))
        .border_style(Style::new().fg(Color::Cyan));
    let inner = block.inner(popup);
    frame.render_widget(block, popup);

    let [heading, fields, hint] = Layout::vertical([
        Constraint::Length(2),
        Constraint::Min(1),
        Constraint::Length(2),
    ])
    .areas(inner);
    let heading_text = format!("Configure {}", editor.action.label());
    frame.render_widget(
        Paragraph::new(heading_text)
            .alignment(Alignment::Center)
            .style(Style::new().fg(Color::Gray)),
        heading,
    );

    let rows = editor
        .fields()
        .into_iter()
        .enumerate()
        .map(|(index, (label, value))| {
            let selected = index == editor.selected;
            let value_style = if value == "on" {
                Style::new().fg(Color::Green).add_modifier(Modifier::BOLD)
            } else if value == "off" {
                Style::new().fg(Color::DarkGray)
            } else {
                Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD)
            };
            Row::new(vec![
                Cell::from(if selected { "›" } else { " " })
                    .style(Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Cell::from(label).style(Style::new().fg(Color::White)),
                Cell::from(value).style(value_style),
            ])
            .style(if selected {
                Style::new().bg(Color::DarkGray)
            } else {
                Style::new()
            })
        });
    frame.render_widget(
        Table::new(
            rows,
            [
                Constraint::Length(3),
                Constraint::Percentage(68),
                Constraint::Percentage(32),
            ],
        ),
        fields,
    );
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(
                "↑↓",
                Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            ),
            Span::raw(" select  "),
            Span::styled(
                "Space",
                Style::new().fg(Color::Green).add_modifier(Modifier::BOLD),
            ),
            Span::raw(" toggle  "),
            Span::styled(
                "+/-",
                Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            ),
            Span::raw(" jobs  "),
            Span::styled(
                "Enter",
                Style::new().fg(Color::Yellow).add_modifier(Modifier::BOLD),
            ),
            Span::raw(" run  Esc cancel"),
        ]))
        .alignment(Alignment::Center)
        .style(Style::new().fg(Color::Gray)),
        hint,
    );
}

fn draw_version_editor(frame: &mut Frame, area: Rect, editor: &VersionEditor) {
    let popup = centered_rect(58, 31, area);
    frame.render_widget(Clear, popup);
    let block = Block::bordered()
        .title(" Change chart version ")
        .border_style(Style::new().fg(Color::Cyan));
    let inner = block.inner(popup);
    frame.render_widget(block, popup);

    let [heading, input, hint] = Layout::vertical([
        Constraint::Length(2),
        Constraint::Length(2),
        Constraint::Length(1),
    ])
    .areas(inner);
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(
                &editor.chart_name,
                Style::new().fg(Color::White).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("  ({})", editor.namespace),
                Style::new().fg(Color::Gray),
            ),
        ]))
        .alignment(Alignment::Center),
        heading,
    );
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(
                " Version  ",
                Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("{}▏", editor.version),
                Style::new().fg(Color::White),
            ),
        ]))
        .block(
            Block::new()
                .borders(Borders::BOTTOM)
                .border_style(Style::new().fg(Color::DarkGray)),
        ),
        input,
    );
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(
                "Enter",
                Style::new().fg(Color::Green).add_modifier(Modifier::BOLD),
            ),
            Span::styled(" save & reload  ", Style::new().fg(Color::Gray)),
            Span::styled(
                "Esc",
                Style::new()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" cancel", Style::new().fg(Color::DarkGray)),
        ]))
        .alignment(Alignment::Center),
        hint,
    );
}

fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    let [vertical] = Layout::vertical([Constraint::Percentage(height)])
        .flex(ratatui::layout::Flex::Center)
        .areas(area);
    let [horizontal] = Layout::horizontal([Constraint::Percentage(width)])
        .flex(ratatui::layout::Flex::Center)
        .areas(vertical);
    horizontal
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn navigation_bursts_are_limited_until_another_key_is_received() {
        let mut navigation_events = 0;

        for _ in 0..MAX_NAVIGATION_EVENTS_PER_FRAME {
            assert!(should_handle_navigation_event(
                KeyCode::Down,
                &mut navigation_events
            ));
        }
        assert!(!should_handle_navigation_event(
            KeyCode::Down,
            &mut navigation_events
        ));
        assert!(should_handle_navigation_event(
            KeyCode::Char(' '),
            &mut navigation_events
        ));
        assert!(should_handle_navigation_event(
            KeyCode::Char('j'),
            &mut navigation_events
        ));
    }

    #[test]
    fn deploy_all_command_uses_non_interactive_child() {
        let args = command_arguments(Path::new("vesshelm.yaml"), &Action::all(ActionKind::Deploy));
        let args: Vec<_> = args.iter().map(|arg| arg.to_string_lossy()).collect();

        assert_eq!(
            args,
            [
                "--config",
                "vesshelm.yaml",
                "deploy",
                "--no-progress",
                "all",
                "--no-interactive",
                "--dry-run"
            ]
        );
    }

    #[test]
    fn selected_sync_command_only_targets_the_selected_chart() {
        let args = command_arguments(
            Path::new("custom.yaml"),
            &Action::charts(ActionKind::Sync, vec!["nginx".to_string()]),
        );
        let args: Vec<_> = args.iter().map(|arg| arg.to_string_lossy()).collect();

        assert_eq!(
            args,
            ["--config", "custom.yaml", "sync", "--no-progress", "nginx"]
        );
    }

    #[test]
    fn marked_charts_are_passed_to_deploy_in_chart_order() {
        let args = command_arguments(
            Path::new("custom.yaml"),
            &Action::charts(
                ActionKind::Deploy,
                vec!["cert-manager".to_string(), "nginx".to_string()],
            ),
        );
        let args: Vec<_> = args.iter().map(|arg| arg.to_string_lossy()).collect();

        assert_eq!(
            args,
            [
                "--config",
                "custom.yaml",
                "deploy",
                "--no-progress",
                "cert-manager",
                "nginx",
                "--no-interactive",
                "--dry-run"
            ]
        );
    }

    #[test]
    fn marked_charts_override_the_cursor_and_keep_table_order() {
        let mut app = App::new(PathBuf::from("missing-test-config.yaml"));
        let cert_manager = Chart {
            name: "cert-manager".to_string(),
            ..Chart::default()
        };
        let nginx = Chart {
            name: "nginx".to_string(),
            ..Chart::default()
        };
        app.charts = vec![cert_manager, nginx];
        app.chart_state.select(Some(1));
        app.selected_charts.insert("nginx".to_string());
        app.selected_charts.insert("cert-manager".to_string());

        assert_eq!(
            app.action_targets(),
            vec!["cert-manager".to_string(), "nginx".to_string()]
        );
    }

    #[test]
    fn ai_update_command_keeps_marked_charts_and_disables_child_prompts() {
        let args = command_arguments(
            Path::new("custom.yaml"),
            &Action::charts(
                ActionKind::AiUpdate,
                vec!["cert-manager".to_string(), "nginx".to_string()],
            ),
        );
        let args: Vec<_> = args.iter().map(|arg| arg.to_string_lossy()).collect();

        assert_eq!(
            args,
            [
                "--config",
                "custom.yaml",
                "ai-update",
                "--no-progress",
                "cert-manager",
                "nginx",
                "--no-interactive"
            ]
        );
    }

    #[test]
    fn delete_command_is_non_interactive_and_targets_one_chart() {
        let args = command_arguments(
            Path::new("custom.yaml"),
            &Action::charts(ActionKind::Delete, vec!["nginx".to_string()]),
        );
        let args: Vec<_> = args.iter().map(|arg| arg.to_string_lossy()).collect();

        assert_eq!(
            args,
            [
                "--config",
                "custom.yaml",
                "delete",
                "--no-progress",
                "nginx",
                "--no-interactive"
            ]
        );
    }

    #[test]
    fn update_command_checks_all_marked_charts_by_default() {
        let args = command_arguments(
            Path::new("custom.yaml"),
            &Action::charts(
                ActionKind::Update,
                vec!["cert-manager".to_string(), "nginx".to_string()],
            ),
        );
        let args: Vec<_> = args.iter().map(|arg| arg.to_string_lossy()).collect();

        assert_eq!(
            args,
            ["--config", "custom.yaml", "update", "cert-manager", "nginx"]
        );
    }

    #[test]
    fn update_mode_is_an_exclusive_choice() {
        let mut options = ActionOptions::for_kind(ActionKind::Update);
        assert_eq!(
            options.fields(),
            vec![
                ("Check only (dry run)", "on".to_string()),
                ("Apply version updates", "off".to_string()),
                ("Apply and sync", "off".to_string()),
                ("Concurrent jobs", "5".to_string()),
            ]
        );

        options.toggle(1);
        assert_eq!(
            options.fields(),
            vec![
                ("Check only (dry run)", "off".to_string()),
                ("Apply version updates", "on".to_string()),
                ("Apply and sync", "off".to_string()),
                ("Concurrent jobs", "5".to_string()),
            ]
        );

        options.toggle(2);
        assert_eq!(
            options.fields(),
            vec![
                ("Check only (dry run)", "off".to_string()),
                ("Apply version updates", "off".to_string()),
                ("Apply and sync", "on".to_string()),
                ("Concurrent jobs", "5".to_string()),
            ]
        );
    }

    #[test]
    fn update_command_keeps_progress_messages_for_live_logs() {
        let args = command_arguments(
            Path::new("custom.yaml"),
            &Action::charts(ActionKind::Update, vec!["nginx".to_string()]),
        );
        let args: Vec<_> = args.iter().map(|arg| arg.to_string_lossy()).collect();

        assert!(!args.iter().any(|arg| arg == "--no-progress"));
    }

    #[test]
    fn deploy_options_are_forwarded_to_the_cli_child() {
        let mut action = Action::charts(ActionKind::Deploy, vec!["nginx".to_string()]);
        action.options = ActionOptions::Deploy {
            dry_run: false,
            take_ownership: true,
            force: true,
            no_diff: true,
            debug: true,
        };
        let args = command_arguments(Path::new("custom.yaml"), &action);
        let args: Vec<_> = args.iter().map(|arg| arg.to_string_lossy()).collect();

        assert_eq!(
            args,
            [
                "--config",
                "custom.yaml",
                "deploy",
                "--no-progress",
                "nginx",
                "--no-interactive",
                "--take-ownership",
                "--force",
                "--no-diff",
                "--debug"
            ]
        );
    }

    #[test]
    fn deploy_uses_dry_run_until_the_operator_disables_it() {
        let args = command_arguments(
            Path::new("custom.yaml"),
            &Action::charts(ActionKind::Deploy, vec!["nginx".to_string()]),
        );
        let args: Vec<_> = args.iter().map(|arg| arg.to_string_lossy()).collect();

        assert!(args.iter().any(|arg| arg == "--dry-run"));
    }

    #[test]
    fn deploy_option_constraints_keep_dry_run_compatible() {
        let mut options = ActionOptions::for_kind(ActionKind::Deploy);
        options.toggle(2);
        options.toggle(3);
        options.toggle(0);

        assert!(matches!(
            options,
            ActionOptions::Deploy {
                dry_run: true,
                force: false,
                no_diff: false,
                ..
            }
        ));
    }

    #[test]
    fn deploy_defaults_to_dry_run() {
        assert!(matches!(
            ActionOptions::for_kind(ActionKind::Deploy),
            ActionOptions::Deploy { dry_run: true, .. }
        ));
    }

    #[test]
    fn option_editor_runs_directly_without_a_second_confirmation() {
        let mut app = App::new(PathBuf::from("missing-test-config.yaml"));
        app.option_editor = Some(OptionEditor::new(Action::all(ActionKind::Add)));

        app.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE))
            .expect("run configured action");

        assert!(app.confirmation.is_none());
        assert!(app.interactive_action.is_some());
    }

    #[test]
    fn version_editor_saves_the_focused_chart_and_reloads_it() {
        use std::io::Write;

        let mut config = tempfile::NamedTempFile::new().expect("create test config");
        config
            .write_all(
                br#"repositories: []
destinations: []
charts:
  - name: nginx
    namespace: default
    version: 1.25.0
"#,
            )
            .expect("write test config");
        let mut app = App::new(config.path().to_path_buf());

        app.handle_key(KeyEvent::new(KeyCode::Char('e'), KeyModifiers::NONE))
            .expect("open version editor");
        app.version_editor
            .as_mut()
            .expect("version editor should open")
            .version = "1.26.0".to_string();
        app.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE))
            .expect("save version");

        assert!(app.version_editor.is_none());
        assert_eq!(app.charts[0].version.as_deref(), Some("1.26.0"));
        let written = std::fs::read_to_string(config.path()).expect("read saved config");
        assert!(written.contains("version: 1.26.0"));
    }

    #[test]
    fn update_and_ai_options_are_forwarded_to_the_cli_child() {
        let mut update = Action::charts(ActionKind::Update, vec!["nginx".to_string()]);
        update.options = ActionOptions::Update {
            mode: UpdateMode::ApplySync,
            jobs: 8,
        };
        let update_args = command_arguments(Path::new("custom.yaml"), &update);
        let update_args: Vec<_> = update_args
            .iter()
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect();
        assert!(update_args.ends_with(&[
            "--apply-sync".to_string(),
            "--jobs".to_string(),
            "8".to_string(),
        ]));

        let mut ai_update = Action::charts(ActionKind::AiUpdate, vec!["nginx".to_string()]);
        ai_update.options = ActionOptions::AiUpdate {
            re_analyze: true,
            jobs: 2,
            debug_ai: true,
        };
        let ai_args = command_arguments(Path::new("custom.yaml"), &ai_update);
        let ai_args: Vec<_> = ai_args
            .iter()
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect();
        assert!(ai_args.ends_with(&[
            "--re-analyze".to_string(),
            "--jobs".to_string(),
            "2".to_string(),
            "--debug-ai".to_string(),
            "--no-interactive".to_string()
        ]));
    }

    #[test]
    fn search_filters_charts_by_namespace_without_changing_source_order() {
        let mut app = App::new(PathBuf::from("missing-test-config.yaml"));
        let cert_manager = Chart {
            name: "cert-manager".to_string(),
            namespace: "cert-manager".to_string(),
            ..Chart::default()
        };
        let nginx = Chart {
            name: "nginx".to_string(),
            namespace: "ingress".to_string(),
            ..Chart::default()
        };
        app.charts = vec![cert_manager, nginx];
        app.search_query = "ingress".to_string();

        assert_eq!(app.visible_chart_indices(), vec![1]);
    }

    #[test]
    fn search_resets_cursor_to_the_first_matching_chart() {
        let mut app = App::new(PathBuf::from("missing-test-config.yaml"));
        let cert_manager = Chart {
            name: "cert-manager".to_string(),
            ..Chart::default()
        };
        let nginx = Chart {
            name: "nginx".to_string(),
            ..Chart::default()
        };
        app.charts = vec![cert_manager, nginx];
        app.chart_state.select(Some(1));
        app.search_query = "cert".to_string();
        app.reset_visible_selection();

        assert_eq!(
            app.selected_chart().map(|chart| chart.name.as_str()),
            Some("cert-manager")
        );
    }

    #[test]
    fn run_status_uses_a_distinct_label_for_cancellation() {
        assert_eq!(RunStatus::Canceled.label(), "canceled");
    }

    #[test]
    fn ansi_log_lines_keep_colors_and_remove_escape_sequences() {
        let line =
            parse_ansi_log_line("\u{1b}[1;93mOutdated\u{1b}[0m nginx \u{1b}[92m1.2.3\u{1b}[0m");

        assert_eq!(line.spans.len(), 3);
        assert_eq!(line.spans[0].content, "Outdated");
        assert_eq!(
            line.spans[0].style,
            Style::new()
                .fg(Color::LightYellow)
                .add_modifier(Modifier::BOLD)
        );
        assert_eq!(line.spans[1].content, " nginx ");
        assert_eq!(line.spans[1].style, Style::new());
        assert_eq!(line.spans[2].content, "1.2.3");
        assert_eq!(line.spans[2].style, Style::new().fg(Color::LightGreen));
    }

    #[test]
    fn ansi_log_lines_support_extended_colors() {
        let line = parse_ansi_log_line("\u{1b}[38;5;208mwarning\u{1b}[0m");

        assert_eq!(line.spans.len(), 1);
        assert_eq!(line.spans[0].content, "warning");
        assert_eq!(line.spans[0].style, Style::new().fg(Color::Indexed(208)));
    }

    #[test]
    fn full_screen_log_view_toggles_without_affecting_the_dashboard_state() {
        let mut app = App::new(PathBuf::from("missing-test-config.yaml"));
        let open = KeyEvent::new(KeyCode::Char('l'), KeyModifiers::NONE);
        let close = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);

        assert!(!app.fullscreen_logs);
        assert!(!app.handle_key(open).expect("open log view"));
        assert!(app.fullscreen_logs);
        assert!(!app.handle_key(close).expect("close log view"));
        assert!(!app.fullscreen_logs);
    }

    #[test]
    fn pod_tracking_uses_helm_release_labels_and_reports_container_status() {
        let mut selectors = release_selectors("edge");
        selectors.extend(
            manifest_pod_selectors(
                "kind: StatefulSet\nspec:\n  selector:\n    matchLabels:\n      app: forgejo-runner\n",
            )
            .expect("parse rendered Helm manifest"),
        );
        let target = PodTarget {
            chart: "forgejo".to_string(),
            namespace: "webapps".to_string(),
            release: "edge".to_string(),
            selectors,
            selectors_resolved: true,
        };

        let pods = parse_pods(
            &[target],
            br#"{"items":[{"metadata":{"name":"edge-7f9","labels":{"app.kubernetes.io/instance":"edge"}},"status":{"phase":"Running","containerStatuses":[{"ready":true,"restartCount":2},{"ready":false,"restartCount":1}]}},{"metadata":{"name":"edge-runner","labels":{"app":"forgejo-runner"}},"status":{"phase":"Pending","containerStatuses":[]}},{"metadata":{"name":"unrelated","labels":{"app":"other"}},"status":{"phase":"Failed","containerStatuses":[]}}]}"#,
        )
        .expect("parse Kubernetes Pods response");
        assert_eq!(pods.len(), 2);
        assert_eq!(pods[0].name, "edge-7f9");
        assert_eq!(pods[0].ready, "1/2");
        assert_eq!(pods[0].restarts, 3);
        assert_eq!(pods[1].name, "edge-runner");
        assert_eq!(pods[1].phase, "Pending");
    }

    #[test]
    fn pod_age_formats_kubernetes_timestamps() {
        assert_eq!(
            kubernetes_timestamp("1970-01-02T00:00:00Z"),
            Some(UNIX_EPOCH + Duration::from_secs(86_400))
        );
        assert_eq!(format_age(Duration::from_secs(5)), "5s");
        assert_eq!(format_age(Duration::from_secs(120)), "2m");
        assert_eq!(format_age(Duration::from_secs(7_200)), "2h");
        assert_eq!(format_age(Duration::from_secs(172_800)), "2d");
    }

    #[test]
    fn sync_state_compares_the_synced_version_to_the_desired_version() {
        let chart = Chart {
            name: "forgejo".to_string(),
            repo_name: Some("forgejo".to_string()),
            version: Some("11.1.0".to_string()),
            ..Chart::default()
        };
        let synced_versions = HashMap::from([(
            "forgejo".to_string(),
            HashMap::from([("forgejo".to_string(), "11.0.0".to_string())]),
        )]);

        assert_eq!(
            sync_state(&chart, &synced_versions),
            SyncState::Pending("11.0.0 → 11.1.0".to_string())
        );
    }

    #[test]
    fn non_deployment_actions_keep_tracking_the_last_deployment_pods() {
        let mut app = App::new(PathBuf::from("missing-test-config.yaml"));
        app.charts = vec![Chart {
            name: "forgejo".to_string(),
            ..Chart::default()
        }];
        app.start_pod_tracking(&Action::charts(
            ActionKind::Deploy,
            vec!["forgejo".to_string()],
        ));
        app.start_pod_tracking(&Action::charts(
            ActionKind::Sync,
            vec!["forgejo".to_string()],
        ));

        assert_eq!(app.pod_targets.len(), 1);
        assert_eq!(app.pod_targets[0].chart, "forgejo");
    }

    #[test]
    fn update_output_exposes_the_available_chart_version() {
        assert_eq!(
            available_version("checking forgejo... Outdated 11.0.0 -> 11.1.0"),
            Some(("forgejo".to_string(), "11.1.0".to_string()))
        );
        assert_eq!(available_version("checking forgejo... Up to date"), None);
    }
}
