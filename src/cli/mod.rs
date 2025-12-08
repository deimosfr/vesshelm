pub mod commands;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "vesshelm")]
#[command(version)]
#[command(about = "A Helm chart management tool", long_about = None)]
pub struct Cli {
    /// Command to execute. Omit it to start the interactive dashboard.
    #[command(subcommand)]
    pub command: Option<commands::Commands>,

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

#[cfg(test)]
mod tests {
    use clap::Parser;

    use super::Cli;

    #[test]
    fn no_subcommand_starts_the_dashboard() {
        let cli = Cli::try_parse_from(["vesshelm"]).expect("CLI should parse without a subcommand");

        assert!(cli.command.is_none());
    }
}
