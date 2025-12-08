use thiserror::Error;

#[derive(Error, Debug)]
pub enum LockError {
    #[error("Failed to parse lockfile: {0}")]
    ParseFailed(#[from] serde_yaml_ng::Error),
    #[error("Failed to write lockfile: {0}")]
    FileWriteFailed(#[from] std::io::Error),
}
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize, Serialize)]
pub struct Lockfile {
    #[serde(default)]
    pub charts: Vec<SyncedChart>,
    #[serde(skip)]
    path: PathBuf,
}

impl Default for Lockfile {
    fn default() -> Self {
        Self {
            charts: Vec::new(),
            path: PathBuf::from("vesshelm.lock"),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SyncedChart {
    pub name: String,
    pub repo_name: String,
    pub version: String,
}

impl Lockfile {
    pub fn load(path: &Path) -> Result<Self, LockError> {
        if path.exists() {
            let content = fs::read_to_string(path).unwrap_or_default();
            let mut lockfile: Lockfile =
                serde_yaml_ng::from_str(&content).map_err(LockError::ParseFailed)?;
            lockfile.path = path.to_path_buf();
            Ok(lockfile)
        } else {
            Ok(Lockfile {
                charts: Vec::new(),
                path: path.to_path_buf(),
            })
        }
    }

    pub fn save(&self) -> Result<(), LockError> {
        let content = serde_yaml_ng::to_string(self).map_err(LockError::ParseFailed)?;
        fs::write(&self.path, content).map_err(LockError::FileWriteFailed)?;
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

    pub fn remove(&mut self, name: &str, repo_name: &str) {
        self.charts
            .retain(|c| !(c.name == name && c.repo_name == repo_name));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_load_nonexistent_returns_default() {
        let lockfile = Lockfile::load(Path::new("/tmp/nonexistent-lockfile.lock")).unwrap();
        assert!(lockfile.charts.is_empty());
    }

    #[test]
    fn test_save_and_reload() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path().to_path_buf();

        let mut lockfile = Lockfile::load(&path).unwrap();
        lockfile.update("nginx".into(), "stable".into(), "1.0.0".into());
        lockfile.save().unwrap();

        let reloaded = Lockfile::load(&path).unwrap();
        assert_eq!(reloaded.charts.len(), 1);
        assert_eq!(reloaded.charts[0].name, "nginx");
        assert_eq!(reloaded.charts[0].version, "1.0.0");
    }

    #[test]
    fn test_update_existing_chart() {
        let mut lockfile = Lockfile::default();
        lockfile.update("nginx".into(), "stable".into(), "1.0.0".into());
        lockfile.update("nginx".into(), "stable".into(), "2.0.0".into());
        assert_eq!(lockfile.charts.len(), 1);
        assert_eq!(lockfile.charts[0].version, "2.0.0");
    }

    #[test]
    fn test_get_chart() {
        let mut lockfile = Lockfile::default();
        lockfile.update("nginx".into(), "stable".into(), "1.0.0".into());

        assert!(lockfile.get("nginx", "stable").is_some());
        assert_eq!(lockfile.get("nginx", "stable").unwrap().version, "1.0.0");
        assert!(lockfile.get("nginx", "other-repo").is_none());
        assert!(lockfile.get("missing", "stable").is_none());
    }

    #[test]
    fn test_remove_chart() {
        let mut lockfile = Lockfile::default();
        lockfile.update("nginx".into(), "stable".into(), "1.0.0".into());
        lockfile.update("redis".into(), "stable".into(), "2.0.0".into());

        lockfile.remove("nginx", "stable");
        assert_eq!(lockfile.charts.len(), 1);
        assert_eq!(lockfile.charts[0].name, "redis");

        // Removing non-existent is no-op
        lockfile.remove("missing", "stable");
        assert_eq!(lockfile.charts.len(), 1);
    }
}
