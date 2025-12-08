mod cli;

use clap::Parser;

use crate::cli::commands::{self, Commands};

#[derive(Parser)]
#[command(name = "vesshelm")]
#[command(version)]
#[command(about = "A Helm chart management tool", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: cli::commands::Commands,

    /// Disable the persistent progress bar
    #[arg(long, global = true)]
    no_progress: bool,

    /// Sets a custom config file
    #[arg(
        short,
        long,
        global = true,
        value_name = "FILE",
        default_value = "vesshelm.yaml"
    )]
    config: String,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let config_path = std::path::Path::new(&cli.config);

    let result = match &cli.command {
        Commands::Init => commands::init::run(config_path).await,
        Commands::Sync(args) => {
            commands::sync::run(args.clone(), cli.no_progress, config_path).await
        }
        Commands::Validate => commands::validate::run(config_path).await,
        Commands::Deploy(args) => {
            commands::deploy::run(args.clone(), cli.no_progress, config_path).await
        }
        Commands::Graph => commands::graph::run(config_path).await,
        Commands::Uninstall(args) => commands::uninstall::run(args.clone(), config_path).await,
        Commands::CheckUpdates(args) => {
            commands::check_updates::run(args.clone(), config_path).await
        }
        Commands::Version => {
            println!("vesshelm {}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
    };

    if let Err(e) = result {
        eprintln!("{}", vesshelm::util::error::format_error(&e));
        std::process::exit(1);
    }
}
