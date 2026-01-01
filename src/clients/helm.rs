use anyhow::{Context, Result};
#[cfg(test)]
use mockall::automock;
use std::path::Path;
use std::process::Command;

#[cfg_attr(test, automock)]
pub trait HelmClient {
    fn repo_add(&self, name: &str, url: &str) -> Result<()>;
    fn repo_update(&self) -> Result<()>;
    fn pull(&self, repo: &str, chart: &str, version: &str, dest_dir: &Path) -> Result<()>;
    fn is_plugin_installed(&self, plugin_name: &str) -> Result<bool>;
    fn install_plugin(&self, plugin_name: &str, url: &str, verify: bool) -> Result<()>;
    fn uninstall(&self, name: &str, namespace: &str) -> Result<()>;
}

pub struct RealHelmClient;

impl Default for RealHelmClient {
    fn default() -> Self {
        Self::new()
    }
}

impl RealHelmClient {
    pub fn new() -> Self {
        Self
    }
}

impl HelmClient for RealHelmClient {
    fn repo_add(&self, name: &str, url: &str) -> Result<()> {
        let output = Command::new("helm")
            .arg("repo")
            .arg("add")
            .arg(name)
            .arg(url)
            .output()
            .context("Failed to execute helm repo add")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if stderr.contains("already exists") {
                return Ok(());
            } else {
                anyhow::bail!("Failed to add helm repo {}: {}", name, stderr);
            }
        }
        Ok(())
    }

    fn repo_update(&self) -> Result<()> {
        let output = Command::new("helm")
            .arg("repo")
            .arg("update")
            .output()
            .context("Failed to execute helm repo update")?;

        if !output.status.success() {
            anyhow::bail!(
                "Failed to update helm repos: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
        Ok(())
    }

    fn pull(&self, repo: &str, chart: &str, version: &str, dest_dir: &Path) -> Result<()> {
        let mut cmd = Command::new("helm");
        cmd.arg("pull")
            .arg(format!("{}/{}", repo, chart))
            .arg("--version")
            .arg(version)
            .arg("--untar")
            .arg("--untardir")
            .arg(dest_dir);

        let status = cmd.output().context("Failed to pull chart")?;
        if !status.status.success() {
            anyhow::bail!(
                "Failed to pull chart {}: {}",
                chart,
                String::from_utf8_lossy(&status.stderr)
            );
        }
        Ok(())
    }

    fn is_plugin_installed(&self, plugin_name: &str) -> Result<bool> {
        let output = Command::new("helm")
            .arg("plugin")
            .arg("list")
            .output()
            .context("Failed to execute helm plugin list")?;

        if !output.status.success() {
            anyhow::bail!(
                "Failed to list helm plugins: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        eprintln!("DEBUG: helm plugin list output:\n{}", stdout);
        Ok(stdout.lines().any(|line| line.starts_with(plugin_name)))
    }

    fn install_plugin(&self, plugin_name: &str, url: &str, verify: bool) -> Result<()> {
        let mut cmd = Command::new("helm");
        cmd.arg("plugin").arg("install");

        if !verify {
            cmd.arg("--verify=false");
        }

        cmd.arg(url);

        let output = cmd
            .output()
            .context(format!("Failed to install helm plugin {}", plugin_name))?;

        if !output.status.success() {
            anyhow::bail!(
                "Failed to install helm plugin {}: {}",
                plugin_name,
                String::from_utf8_lossy(&output.stderr)
            );
        }
        Ok(())
    }

    fn uninstall(&self, name: &str, namespace: &str) -> Result<()> {
        let output = Command::new("helm")
            .arg("uninstall")
            .arg(name)
            .arg("-n")
            .arg(namespace)
            .output()
            .context("Failed to execute helm uninstall")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if stderr.contains("release: not found") {
                return Ok(());
            }
            anyhow::bail!("Failed to uninstall release {}: {}", name, stderr);
        }
        Ok(())
    }
}
