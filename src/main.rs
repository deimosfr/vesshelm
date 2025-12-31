mod cli;

use clap::Parser;

use crate::cli::commands::{self, Commands};

#[tokio::main]
async fn main() {
    let cli = cli::Cli::parse();
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
            commands::check_updates::run(args.clone(), cli.no_progress, config_path).await
        }
        Commands::Version => {
            println!("vesshelm {}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
        Commands::Completion(args) => commands::completion::completion(args),
        Commands::Add => commands::add::run(config_path).await,
    };

    if let Err(e) = result {
        eprintln!("{}", vesshelm::util::error::format_error(&e));
        std::process::exit(1);
    }
}
