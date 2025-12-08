pub mod check_updates;
pub mod deploy;
pub mod graph;
pub mod init;
pub mod sync;
pub mod uninstall;
pub mod validate;

use clap::{Args, Subcommand};

#[derive(Subcommand)]
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
    CheckUpdates(CheckUpdatesArgs),
    /// Display the application version
    Version,
}

#[derive(Args, Clone, Debug)]
pub struct DeployArgs {
    /// Perform a dry run (execute helm diff only)
    #[clap(long)]
    pub dry_run: bool,

    /// Only deploy the specified charts (comma-separated)
    /// Only deploy the specified charts (comma-separated)
    #[clap(long, value_delimiter = ',')]
    pub only: Option<Vec<String>>,

    /// Skip interactive confirmation
    #[clap(long)]
    pub no_interactive: bool,

    /// Take ownership of existing resources
    #[clap(long)]
    pub take_ownership: bool,

    /// Force deployment even if no changes are detected
    #[clap(long, short = 'f', conflicts_with = "dry_run")]
    pub force: bool,
}

#[derive(Args, Clone, Debug)]
pub struct SyncArgs {
    /// Only sync the specified charts (comma-separated)
    #[clap(long, value_delimiter = ',')]
    pub only: Option<Vec<String>>,

    /// Force sync even if charts are already up to date
    #[clap(long)]
    pub ignore_skip: bool,
}

#[derive(Args, Clone, Debug)]
pub struct UninstallArgs {
    /// The name of the chart to uninstall
    #[arg(required = true)]
    pub name: String,

    /// Skip interactive confirmation
    #[clap(long)]
    pub no_interactive: bool,
}

#[derive(Args, Clone, Debug)]
pub struct CheckUpdatesArgs {
    /// Update vesshelm.yaml with the new versions
    #[clap(long)]
    pub apply: bool,

    /// Only check specified charts (comma-separated)
    #[clap(long, value_delimiter = ',')]
    pub only: Option<Vec<String>>,
}
