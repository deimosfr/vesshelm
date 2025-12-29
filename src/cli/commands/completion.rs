use super::CompletionArgs;

use anyhow::Result;
use clap::CommandFactory;
use clap_complete::generate;
use std::io;

pub fn completion(args: &CompletionArgs) -> Result<()> {
    let mut cmd = crate::cli::Cli::command();
    let bin_name = cmd.get_name().to_string();
    generate(args.shell, &mut cmd, bin_name, &mut io::stdout());
    Ok(())
}
