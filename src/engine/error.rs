use thiserror::Error;
use walkdir::Error as WalkDirError;

#[derive(Error, Debug)]
pub enum SyncError {
    #[error("Failed to deliver sync events: {0}")]
    ObserverFailed(#[from] tokio::task::JoinError),

    #[error("Repository '{0}' not found in configuration")]
    RepoNotFound(String),

    #[error("Version required for chart '{0}'")]
    VersionRequired(String),

    #[error(transparent)]
    Config(#[from] crate::config::error::ConfigError),

    #[error("Failed to create destination directory '{0}': {1}")]
    DestDirCreation(String, std::io::Error),

    #[error("Failed to create temporary directory: {0}")]
    TempDirCreation(std::io::Error),

    #[error("Chart path required for git repository")]
    ChartPathRequired,

    #[error("Chart not found at '{0}' after pull")]
    ChartNotFoundAfterPull(String),

    #[error("Failed to remove existing chart directory: {0}")]
    RemoveExistingFailed(std::io::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Walkdir error: {0}")]
    Walkdir(#[from] WalkDirError),

    #[error("Copy failed: {0}")]
    CopyFailed(std::io::Error),

    #[error(transparent)]
    Helm(#[from] crate::clients::error::HelmClientError),

    #[error(transparent)]
    Git(#[from] crate::clients::error::GitError),
}
