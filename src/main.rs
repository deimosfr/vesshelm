use clap::Parser;

use vesshelm::cli::{
    self,
    commands::{self, Commands},
};

#[tokio::main]
async fn main() {
    let cli = cli::Cli::parse();
    let config_path = std::path::Path::new(&cli.config);

    // Keep terminal setup and teardown wholly inside the dashboard. In
    // particular, this avoids cancelling its restoration path on Ctrl+C.
    if cli.command.is_none() {
        if let Err(error) = vesshelm::tui::run(config_path).await {
            eprintln!("{}", vesshelm::util::error::format_error(&error));
            std::process::exit(1);
        }
        return;
    }

    let command_future = async {
        match &cli.command {
            None => unreachable!("the dashboard is handled before command dispatch"),
            Some(Commands::Init) => commands::init::run(config_path).await,
            Some(Commands::Sync(args)) => {
                commands::sync::run(args.clone(), cli.no_progress, config_path).await
            }
            Some(Commands::Validate) => commands::validate::run(config_path).await,
            Some(Commands::Deploy(args)) => {
                commands::deploy::run(
                    args.clone(),
                    cli.no_progress,
                    config_path,
                    &vesshelm::util::interaction::TerminalInteraction,
                )
                .await
            }
            Some(Commands::Graph) => commands::graph::run(config_path).await,
            Some(Commands::Uninstall(args)) => {
                commands::uninstall::run(
                    args.clone(),
                    config_path,
                    &vesshelm::util::interaction::TerminalInteraction,
                )
                .await
            }
            Some(Commands::Update(args)) => {
                commands::update::run(
                    args.clone(),
                    cli.no_progress,
                    config_path,
                    vesshelm::clients::helm::RealHelmClient::new(),
                )
                .await
            }
            Some(Commands::Version) => {
                println!("vesshelm {}", env!("CARGO_PKG_VERSION"));
                Ok(())
            }
            Some(Commands::Completion(args)) => commands::completion::completion(args),
            Some(Commands::Add) => {
                commands::add::run(
                    config_path,
                    &vesshelm::util::interaction::TerminalInteraction,
                )
                .await
            }
            Some(Commands::Delete(args)) => {
                commands::delete::run(
                    args.clone(),
                    config_path,
                    &vesshelm::util::interaction::TerminalInteraction,
                )
                .await
            }
            Some(Commands::AiUpdate(args)) => {
                commands::ai_update::run(
                    args.clone(),
                    cli.no_progress,
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
            Err(vesshelm::error::VesshelmError::CommandFailed("Interrupted by Ctrl+C".to_string()))
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
            Err(vesshelm::error::VesshelmError::CommandFailed("Interrupted by SIGTERM".to_string()))
        }
    };

    if let Err(e) = result {
        eprintln!("{}", vesshelm::util::error::format_error(&e));
        std::process::exit(1);
    }
}
