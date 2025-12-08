use regex::Error as RegexError;
use thiserror::Error;
use validator::ValidationErrors;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Failed to read configuration file '{0}': {1}")]
    FileReadFailed(String, #[source] std::io::Error),

    #[error("Failed to parse configuration YAML from '{1}': {0}")]
    ParseFailed(#[source] serde_yaml_ng::Error, String),

    #[error("Failed to parse configuration YAML: {0}\nHint: {1} (file: {2})")]
    ParseFailedWithHint(#[source] serde_yaml_ng::Error, String, String),

    #[error("Configuration validation failed:\n{0}")]
    ValidationFailed(#[from] ValidationErrors),

    #[error("Default destination 'default' not found in configuration (chart: {0})")]
    DefaultDestNotFound(String),

    #[error("Chart '{0}' not found in configuration")]
    ChartNotFound(String),

    #[error("Regex error: {0}")]
    Regex(#[from] RegexError),
    #[error("Regex capture failed for pattern '{0}'")]
    RegexCaptureFailed(String),

    #[error("Version boundary crossed during update (chart: {0})")]
    VersionBoundaryCrossed(String),

    #[error("Version field not found for chart '{0}'")]
    VersionFieldNotFound(String),

    #[error("Failed to write configuration file '{0}': {1}")]
    FileWriteFailed(String, #[source] std::io::Error),
}
