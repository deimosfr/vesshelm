use crate::clients::helm::{HelmClient, RealHelmClient};
use crate::error::VesshelmError;
use crate::util::ui;
use console::style;
use std::fs;
use std::path::Path;

pub async fn run(config_path: &Path) -> Result<(), VesshelmError> {
    ui::print_step("Vesshelm initialization");

    // Check for helm
    if which::which("helm").is_err() {
        return Err(VesshelmError::MissingDependency(
            "helm executable not found in PATH. Please install Helm to use Vesshelm.".to_string(),
        ));
    }

    // Check for helm-diff plugin
    let helm_client = RealHelmClient::new();
    if !helm_client
        .is_plugin_installed("diff")
        .map_err(crate::clients::error::ClientError::from)?
    {
        println!("Installing helm-diff plugin...");
        helm_client
            .install_plugin("diff", "https://github.com/databus23/helm-diff", false)
            .map_err(crate::clients::error::ClientError::from)?;
        println!("Successfully installed helm-diff plugin.");
    } else {
        println!("helm-diff plugin is already installed.");
    }

    // Create vesshelm.yaml (or custom path)
    if config_path.exists() {
        println!(
            "{} vesshelm.yaml already exists.",
            style("Skipped:").yellow().bright()
        );
    } else {
        let default_content = r#"charts: []

destinations:
  - name: default
    path: ./charts

vesshelm:
  helm_args: "upgrade --install {{ name }} {{ destination }}/{{ name }} -n {{ namespace }} --wait --rollback-on-failure --create-namespace"
  diff_enabled: true
  diff_args: "diff upgrade --suppress-secrets --allow-unreleased {{ name }} {{ destination }} -n {{ namespace }}"

variables_files: []
secrets_files: []

repositories:
  - name: stable
    url: https://charts.helm.sh/stable
"#;
        fs::write(config_path, default_content)?;
        println!(
            "{} Initialized empty vesshelm.yaml",
            style("Success:").green().bright()
        );
    }

    Ok(())
}
