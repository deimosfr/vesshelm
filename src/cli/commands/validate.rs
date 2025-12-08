use crate::config::Config;
use crate::error::VesshelmError;
use crate::util::ui;
use console::style;
use std::path::Path;

pub async fn run(config_path: &Path) -> Result<(), VesshelmError> {
    ui::print_step("Validating configuration");

    Config::load_from_path(config_path)?;

    println!("{}", style("Validation successful!").green().bright());
    Ok(())
}
