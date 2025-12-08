use crate::error::VesshelmError;
use git2::{DiffFormat, DiffLine, DiffOptions, Repository, StatusOptions};
use std::path::Path;

pub struct GitUtil {
    repo: Repository,
}

impl GitUtil {
    pub fn discover<P: AsRef<Path>>(path: P) -> Result<Self, VesshelmError> {
        let repo = Repository::discover(path).map_err(|e| {
            VesshelmError::CommandFailed(format!("Failed to discover git repository: {}", e))
        })?;
        Ok(Self { repo })
    }

    pub fn repo_root(&self) -> &Path {
        self.repo.workdir().unwrap_or_else(|| Path::new("."))
    }

    pub fn is_dirty(&self) -> Result<bool, VesshelmError> {
        let mut status_opts = StatusOptions::new();
        status_opts.include_untracked(true);
        let statuses = self.repo.statuses(Some(&mut status_opts)).map_err(|e| {
            VesshelmError::CommandFailed(format!("Failed to get git status: {}", e))
        })?;

        Ok(!statuses.is_empty())
    }

    pub fn read_file_from_head(&self, path: &Path) -> Result<String, VesshelmError> {
        let head = self
            .repo
            .head()
            .map_err(|e| VesshelmError::CommandFailed(format!("Failed to get HEAD: {}", e)))?;
        let tree = head.peel_to_tree().map_err(|e| {
            VesshelmError::CommandFailed(format!("Failed to peel HEAD to tree: {}", e))
        })?;

        // Git2 expects paths relative to the repo root.
        let entry = tree.get_path(path).map_err(|e| {
            VesshelmError::CommandFailed(format!("Failed to find {:?} in HEAD: {}", path, e))
        })?;
        let obj = entry.to_object(&self.repo).map_err(|e| {
            VesshelmError::CommandFailed(format!("Failed to get object for {:?}: {}", path, e))
        })?;
        let blob = obj
            .as_blob()
            .ok_or_else(|| VesshelmError::CommandFailed(format!("{:?} is not a blob", path)))?;

        let content = std::str::from_utf8(blob.content()).map_err(|e| {
            VesshelmError::CommandFailed(format!("Failed to parse {:?} as UTF-8: {}", path, e))
        })?;

        Ok(content.to_string())
    }

    pub fn get_diff(&self, from_head: bool) -> Result<String, VesshelmError> {
        let mut diff_opts = DiffOptions::new();

        let diff = if from_head {
            let head = self
                .repo
                .head()
                .map_err(|e| VesshelmError::CommandFailed(format!("Failed to get HEAD: {}", e)))?;
            let tree = head.peel_to_tree().map_err(|e| {
                VesshelmError::CommandFailed(format!("Failed to peel HEAD to tree: {}", e))
            })?;
            self.repo
                .diff_tree_to_workdir_with_index(Some(&tree), Some(&mut diff_opts))
                .map_err(|e| {
                    VesshelmError::CommandFailed(format!(
                        "Failed to generate diff from HEAD: {}",
                        e
                    ))
                })?
        } else {
            self.repo
                .diff_index_to_workdir(None, Some(&mut diff_opts))
                .map_err(|e| {
                    VesshelmError::CommandFailed(format!(
                        "Failed to generate diff from index: {}",
                        e
                    ))
                })?
        };

        let mut diff_text = String::new();
        diff.print(DiffFormat::Patch, |_delta, _hunk, line: DiffLine| {
            let origin = line.origin();
            if origin == '+' || origin == '-' || origin == ' ' {
                diff_text.push(origin);
            }
            diff_text.push_str(std::str::from_utf8(line.content()).unwrap_or_default());
            true
        })
        .map_err(|e| VesshelmError::CommandFailed(format!("Failed to print diff: {}", e)))?;

        Ok(diff_text)
    }
}
