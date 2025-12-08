use thiserror::Error;

#[derive(Error, Debug)]
pub enum VesshelmError {
    #[error("Configuration error: {0}")]
    ConfigAndValidation(String),

    #[error("Chart {0} not found in repository {1}")]
    ChartNotFound(String, String),

    #[error("Failed to update helm repositories: {0}")]
    RepoUpdateFailed(String),

    #[error("Failed to sync chart {0}: {1}")]
    SyncFailed(String, String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Git error: {0}")]
    Git(#[from] git2::Error),

    #[error("Helm error: {0}")]
    Helm(String),

    #[error("Unknown error: {0}")]
    Unknown(#[from] anyhow::Error),
}
