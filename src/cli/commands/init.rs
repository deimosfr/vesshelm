use anyhow::{Context, Result};
use console::style;
use std::fs;
use std::path::Path;
use vesshelm::clients::helm::{HelmClient, RealHelmClient};

pub async fn run(config_path: &Path) -> Result<()> {
    println!("{} Vesshelm initialization", style("==>").bold().green());

    // Check for helm
    if which::which("helm").is_err() {
        anyhow::bail!(
            "Error: helm executable not found in PATH. Please install Helm to use Vesshelm."
        );
    }

    // Check for helm-diff plugin
    let helm_client = RealHelmClient::new();
    if !helm_client.is_plugin_installed("diff")? {
        println!("Installing helm-diff plugin...");
        helm_client.install_plugin("diff", "https://github.com/databus23/helm-diff", false)?;
        println!("Successfully installed helm-diff plugin.");
    } else {
        println!("helm-diff plugin is already installed.");
    }

    // Create vesshelm.yaml (or custom path)
    if config_path.exists() {
        println!("Configuration file '{:?}' already exists.", config_path);
    } else {
        let default_content = r#"charts: []

destinations:
  - name: default
    path: ./charts

helm:
  args: "upgrade --install {{ name }} {{ destination }}/{{ name }} -n {{ namespace }} --wait --rollback-on-failure --create-namespace"
  diff_enabled: true
  diff_args: "diff upgrade --allow-unreleased {{ name }} {{ destination }} -n {{ namespace }}"
        
repositories:
  - name: stable
    url: https://charts.helm.sh/stable
"#;
        fs::write(config_path, default_content)
            .with_context(|| format!("Failed to write {:?}", config_path))?;
        println!("Created default configuration file '{:?}'.", config_path);
    }

    Ok(())
}
