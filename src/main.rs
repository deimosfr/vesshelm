use clap::Parser;

use vesshelm::cli::{
    self,
    commands::{self, Commands},
};

#[tokio::main]
async fn main() {
    let cli = cli::Cli::parse();
    let config_path = std::path::Path::new(&cli.config);

    let command_future = async {
        match &cli.command {
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
            Commands::Add => {
                commands::add::run(
                    config_path,
                    &vesshelm::util::interaction::TerminalInteraction,
                )
                .await
            }
            Commands::Delete(args) => {
                commands::delete::run(
                    args.clone(),
                    config_path,
                    &vesshelm::util::interaction::TerminalInteraction,
                )
                .await
            }
        }
    };

    let result = tokio::select! {
        res = command_future => res,
        _ = tokio::signal::ctrl_c() => {
            eprintln!("\nReceived Ctrl+C, cleaning up...");
            // Returning error to ensure non-zero exit code
            Err(anyhow::anyhow!("Interrupted by Ctrl+C"))
        }
        _ = async {
            #[cfg(unix)]
            {
                let mut sigterm = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()).unwrap();
                sigterm.recv().await;
            }
            #[cfg(not(unix))]
            {
                std::future::pending::<()>().await;
            }
        } => {
            eprintln!("\nReceived SIGTERM, cleaning up...");
            Err(anyhow::anyhow!("Interrupted by SIGTERM"))
        }
    };

    if let Err(e) = result {
        eprintln!("{}", vesshelm::util::error::format_error(&e));
        std::process::exit(1);
    }
}
