pub mod commands;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "vesshelm")]
#[command(version)]
#[command(about = "A Helm chart management tool", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: commands::Commands,

    /// Disable the persistent progress bar
    #[arg(long, global = true)]
    pub no_progress: bool,

    /// Sets a custom config file
    #[arg(
        short,
        long,
        global = true,
        value_name = "FILE",
        default_value = "vesshelm.yaml"
    )]
    pub config: String,
}
