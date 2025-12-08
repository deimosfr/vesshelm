#[cfg(test)]
use mockall::automock;
use std::path::Path;

use crate::clients::error::GitError;

#[cfg_attr(test, automock)]
pub trait GitClient {
    fn clone(&self, url: &str, path: &Path) -> Result<(), GitError>;
    fn checkout(&self, path: &Path, version: &str) -> Result<(), GitError>;
}

#[derive(Clone, Copy, Debug)]
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
    fn clone(&self, url: &str, path: &Path) -> Result<(), GitError> {
        git2::Repository::clone(url, path)
            .map_err(|e| GitError::CloneFailed(url.to_string(), e))?;
        Ok(())
    }

    fn checkout(&self, path: &Path, version: &str) -> Result<(), GitError> {
        let repository = git2::Repository::open(path)?;
        let (object, reference) = repository
            .revparse_ext(version)
            .map_err(|e| GitError::CheckoutFailed(version.to_string(), e))?;

        repository
            .checkout_tree(&object, None)
            .map_err(|e| GitError::CheckoutFailed(version.to_string(), e))?;

        match reference {
            Some(gref) => {
                let ref_name = gref
                    .name()
                    .map_err(|_| GitError::RefNotFound(version.to_string()))?;
                repository.set_head(ref_name)?;
            }
            None => repository.set_head_detached(object.id())?,
        }
        Ok(())
    }
}
