pub mod add;
pub mod ai_update;
pub mod completion;
pub mod delete;
pub mod deploy;
pub mod graph;
pub mod init;
pub mod sync;
pub mod uninstall;
pub mod update;
pub mod validate;

use clap::{Args, Subcommand};

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Initialize Vesshelm in the current directory
    Init,
    /// Sync charts defined in vesshelm.yaml
    Sync(SyncArgs),
    /// Validate the configuration file
    Validate,
    /// Deploy charts using local helm binary
    Deploy(DeployArgs),
    /// Show the dependency graph of charts
    Graph,
    /// Uninstall a chart release
    Uninstall(UninstallArgs),
    /// Check for updates
    #[clap(visible_alias = "check-update")]
    Update(UpdateArgs),
    /// Display the application version
    Version,
    /// Generate shell completion scripts
    Completion(CompletionArgs),
    /// Add a new chart to the configuration
    Add,
    /// Delete a chart from configuration and filesystem
    Delete(DeleteArgs),
    /// Perform an AI-powered update and safety analysis
    #[clap(visible_alias = "au")]
    AiUpdate(AiUpdateArgs),
}

#[derive(Args, Clone, Debug)]
pub struct CompletionArgs {
    /// The shell to generate completion for
    #[clap(value_enum)]
    pub shell: clap_complete::Shell,
}

#[derive(Args, Clone, Debug)]
pub struct DeployArgs {
    /// Perform a dry run (execute helm diff only)
    #[clap(short = 'D', long)]
    pub dry_run: bool,

    /// Only deploy the specified charts (or 'all' for all charts)
    pub charts: Option<Vec<String>>,

    /// Skip interactive confirmation
    #[clap(short = 'i', long)]
    pub no_interactive: bool,

    /// Take ownership of existing resources
    #[clap(short = 'o', long)]
    pub take_ownership: bool,

    /// Force deployment even if no changes are detected
    #[clap(short = 'f', long, conflicts_with = "dry_run")]
    pub force: bool,

    /// Disable diff view even if enabled in configuration
    #[clap(short = 'r', long, conflicts_with = "dry_run")]
    pub no_diff: bool,

    /// Enable verbose debug output for Helm commands
    #[clap(short = 'd', long)]
    pub debug: bool,
}

#[derive(Args, Clone, Debug)]
pub struct SyncArgs {
    /// Only sync the specified charts
    pub charts: Option<Vec<String>>,

    /// Force sync (ignore skip conditions)
    #[clap(short = 's', long)]
    pub ignore_skip: bool,

    /// Number of concurrent jobs to run (default: 5)
    #[clap(short = 'j', long, default_value_t = 5)]
    pub jobs: usize,
}

#[derive(Args, Clone, Debug)]
pub struct UninstallArgs {
    /// The name of the chart to uninstall
    pub name: Option<String>,

    /// Skip interactive confirmation
    #[clap(short = 'i', long)]
    pub no_interactive: bool,
}

#[derive(Args, Clone, Debug)]
pub struct UpdateArgs {
    /// Update vesshelm.yaml with the new versions
    #[clap(short = 'a', long)]
    pub apply: bool,

    /// Update vesshelm.yaml and sync changes immediately
    #[clap(short = 'S', long)]
    pub apply_sync: bool,

    /// Only check specified charts
    pub charts: Option<Vec<String>>,

    /// Number of concurrent jobs to run (default: 5)
    #[clap(short = 'j', long, default_value_t = 5)]
    pub jobs: usize,
}

#[derive(Args, Clone, Debug)]
pub struct DeleteArgs {
    /// The name of the chart to delete
    pub name: Option<String>,

    /// Skip interactive confirmation
    #[clap(short = 'i', long)]
    pub no_interactive: bool,
}

#[derive(Args, Clone, Debug)]
pub struct AiUpdateArgs {
    /// Re-analyze current local changes without syncing new versions
    #[clap(short = 'R', long)]
    pub re_analyze: bool,

    /// Only check specified charts
    pub charts: Option<Vec<String>>,

    /// Number of concurrent jobs to run (default: 5)
    #[clap(short = 'j', long, default_value_t = 5)]
    pub jobs: usize,

    /// Debug AI provider: print invocation, stream stderr, dump prompt to stderr
    #[clap(long)]
    pub debug_ai: bool,

    /// Disable all interactive prompts. Updates all outdated charts (or those listed
    /// in `charts`), fails on a dirty repo, and skips the final deploy step.
    #[clap(short = 'i', long)]
    pub no_interactive: bool,
}
