mod cli;

use clap::Parser;

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
        cli::commands::Commands::Init => cli::commands::init::run(config_path).await,
        cli::commands::Commands::Sync(args) => {
            cli::commands::sync::run(args.clone(), cli.no_progress, config_path).await
        }
        cli::commands::Commands::Validate => cli::commands::validate::run(config_path).await,
        cli::commands::Commands::Deploy(args) => {
            cli::commands::deploy::run(args.clone(), cli.no_progress, config_path).await
        }
        cli::commands::Commands::Graph => cli::commands::graph::run(config_path).await,
        cli::commands::Commands::Uninstall(args) => {
            cli::commands::uninstall::run(args.clone(), config_path).await
        }
        cli::commands::Commands::CheckUpdates(args) => {
            cli::commands::check_updates::run(args.clone(), config_path).await
        }
        cli::commands::Commands::Version => {
            println!("vesshelm {}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
    };

    if let Err(e) = result {
        eprintln!("{}", vesshelm::util::error::format_error(&e));
        std::process::exit(1);
    }
}
