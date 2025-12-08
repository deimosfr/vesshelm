use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct Lockfile {
    #[serde(default)]
    pub charts: Vec<SyncedChart>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SyncedChart {
    pub name: String,
    pub repo_name: String,
    pub version: String,
}

impl Lockfile {
    pub fn load() -> Result<Self> {
        let path = Path::new("vesshelm.lock");
        if path.exists() {
            let content = fs::read_to_string(path).unwrap_or_default();
            serde_yaml_ng::from_str(&content).context("Failed to parse vesshelm.lock")
        } else {
            Ok(Lockfile::default())
        }
    }

    pub fn save(&self) -> Result<()> {
        let content = serde_yaml_ng::to_string(self).context("Failed to serialize vesshelm.lock")?;
        fs::write("vesshelm.lock", content).context("Failed to write vesshelm.lock")?;
        Ok(())
    }

    pub fn get(&self, name: &str, repo_name: &str) -> Option<&SyncedChart> {
        self.charts
            .iter()
            .find(|c| c.name == name && c.repo_name == repo_name)
    }

    pub fn update(&mut self, name: String, repo_name: String, version: String) {
        if let Some(existing) = self
            .charts
            .iter_mut()
            .find(|c| c.name == name && c.repo_name == repo_name)
        {
            existing.version = version;
        } else {
            self.charts.push(SyncedChart {
                name,
                repo_name,
                version,
            });
        }
    }
}
