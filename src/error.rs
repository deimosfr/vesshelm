use thiserror::Error;

#[derive(Error, Debug)]
pub enum VesshelmError {
    #[error(transparent)]
    Config(#[from] crate::config::error::ConfigError),
    #[error(transparent)]
    Lock(#[from] crate::lock::LockError),
    #[error(transparent)]
    Client(#[from] crate::clients::error::ClientError),
    #[error(transparent)]
    Variables(#[from] crate::util::error::VariablesError),
    #[error(transparent)]
    Encryption(#[from] crate::util::error::EncryptionError),
    #[error(transparent)]
    Dag(#[from] crate::util::error::DagError),
    #[error(transparent)]
    Filter(#[from] crate::util::error::FilterError),
    #[error(transparent)]
    Interaction(#[from] crate::util::error::InteractionError),
    #[error(transparent)]
    Progress(#[from] crate::util::error::ProgressError),
    #[error(transparent)]
    Merge(#[from] crate::util::helm::MergeError),
    #[error(transparent)]
    Sync(#[from] crate::engine::error::SyncError),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("Missing dependency: {0}")]
    MissingDependency(String),
    #[error("Command failed: {0}")]
    CommandFailed(String),
}
