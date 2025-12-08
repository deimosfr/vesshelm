#[cfg(test)]
use mockall::automock;
use std::path::Path;
use std::process::Command;

use crate::clients::error::HelmClientError;

#[cfg_attr(test, automock)]
pub trait HelmClient {
    fn repo_add(&self, name: &str, url: &str) -> Result<(), HelmClientError>;
    fn repo_update(&self) -> Result<(), HelmClientError>;
    fn pull(&self, chart_ref: &str, version: &str, dest_dir: &Path) -> Result<(), HelmClientError>;
    fn is_plugin_installed(&self, plugin_name: &str) -> Result<bool, HelmClientError>;
    fn install_plugin(
        &self,
        plugin_name: &str,
        url: &str,
        verify: bool,
    ) -> Result<(), HelmClientError>;
    fn show_chart(&self, chart_ref: &str) -> Result<String, HelmClientError>;
    fn uninstall(&self, name: &str, namespace: &str) -> Result<(), HelmClientError>;
    fn search_repo(&self, chart_ref: &str, devel: bool) -> Result<String, HelmClientError>;
}

#[derive(Clone, Copy, Debug)]
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
    fn repo_add(&self, name: &str, url: &str) -> Result<(), HelmClientError> {
        let output = Command::new("helm")
            .arg("repo")
            .arg("add")
            .arg(name)
            .arg(url)
            .output()
            .map_err(HelmClientError::Io)?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if stderr.contains("already exists") {
                return Ok(());
            } else {
                return Err(HelmClientError::RepoAddFailed(
                    name.to_string(),
                    stderr.to_string(),
                ));
            }
        }
        Ok(())
    }

    fn repo_update(&self) -> Result<(), HelmClientError> {
        let status = Command::new("helm")
            .arg("repo")
            .arg("update")
            .status()
            .map_err(HelmClientError::Io)?;

        if !status.success() {
            return Err(HelmClientError::RepoUpdateFailed(
                "Failed to update repositories".to_string(),
            ));
        }
        Ok(())
    }

    fn pull(&self, chart_ref: &str, version: &str, dest_dir: &Path) -> Result<(), HelmClientError> {
        let mut cmd = Command::new("helm");
        cmd.arg("pull")
            .arg(chart_ref)
            .arg("--version")
            .arg(version)
            .arg("--untar")
            .arg("--untardir")
            .arg(dest_dir);

        let status = cmd.output().map_err(HelmClientError::Io)?;
        if !status.status.success() {
            return Err(HelmClientError::PullFailed(
                chart_ref.to_string(),
                String::from_utf8_lossy(&status.stderr).to_string(),
            ));
        }
        Ok(())
    }

    fn is_plugin_installed(&self, plugin_name: &str) -> Result<bool, HelmClientError> {
        let output = Command::new("helm")
            .arg("plugin")
            .arg("list")
            .output()
            .map_err(HelmClientError::Io)?;

        if !output.status.success() {
            return Err(HelmClientError::PluginListFailed(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.lines().any(|line| line.starts_with(plugin_name)))
    }

    fn install_plugin(
        &self,
        plugin_name: &str,
        url: &str,
        verify: bool,
    ) -> Result<(), HelmClientError> {
        let mut cmd = Command::new("helm");
        cmd.arg("plugin").arg("install");

        if !verify {
            cmd.arg("--verify=false");
        }

        cmd.arg(url);

        let output = cmd.output().map_err(HelmClientError::Io)?;

        if !output.status.success() {
            return Err(HelmClientError::PluginInstallFailed(
                plugin_name.to_string(),
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }
        Ok(())
    }

    fn show_chart(&self, chart_ref: &str) -> Result<String, HelmClientError> {
        let output = Command::new("helm")
            .arg("show")
            .arg("chart")
            .arg(chart_ref)
            .output()
            .map_err(HelmClientError::Io)?;

        if !output.status.success() {
            return Err(HelmClientError::ShowChartFailed(
                chart_ref.to_string(),
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    fn uninstall(&self, name: &str, namespace: &str) -> Result<(), HelmClientError> {
        let output = Command::new("helm")
            .arg("uninstall")
            .arg(name)
            .arg("-n")
            .arg(namespace)
            .output()
            .map_err(HelmClientError::Io)?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if stderr.contains("release: not found") {
                return Ok(());
            }
            return Err(HelmClientError::UninstallFailed(
                name.to_string(),
                stderr.to_string(),
            ));
        }
        Ok(())
    }

    fn search_repo(&self, chart_ref: &str, devel: bool) -> Result<String, HelmClientError> {
        let mut cmd = Command::new("helm");
        cmd.arg("search")
            .arg("repo")
            .arg(chart_ref)
            .arg("--output")
            .arg("yaml");
        if devel {
            cmd.arg("--devel");
        }
        let output = cmd.output().map_err(HelmClientError::Io)?;

        if !output.status.success() {
            return Err(HelmClientError::SearchRepoFailed(
                chart_ref.to_string(),
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}
