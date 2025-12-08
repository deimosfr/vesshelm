use anyhow::{Context, Result};
#[cfg(test)]
use mockall::automock;
use std::path::Path;

#[cfg_attr(test, automock)]
pub trait GitClient {
    fn clone(&self, url: &str, path: &Path) -> Result<()>;
    fn checkout(&self, path: &Path, version: &str) -> Result<()>;
}

pub struct RealGitClient;

impl Default for RealGitClient {
    fn default() -> Self {
        Self::new()
    }
}

impl RealGitClient {
    pub fn new() -> Self {
        Self
    }
}

impl GitClient for RealGitClient {
    fn clone(&self, url: &str, path: &Path) -> Result<()> {
        git2::Repository::clone(url, path)
            .map_err(|e| anyhow::anyhow!("Failed to clone git repo {}: {}", url, e))?;
        Ok(())
    }

    fn checkout(&self, path: &Path, version: &str) -> Result<()> {
        let repository = git2::Repository::open(path)?;
        let (object, reference) = repository
            .revparse_ext(version)
            .with_context(|| format!("Failed to find version {}", version))?;

        repository
            .checkout_tree(&object, None)
            .context("Failed to checkout tree")?;

        match reference {
            Some(gref) => {
                let ref_name = gref
                    .name()
                    .ok_or_else(|| anyhow::anyhow!("Failed to get reference name"))?;
                repository.set_head(ref_name)?;
            }
            None => repository.set_head_detached(object.id())?,
        }
        Ok(())
    }
}
