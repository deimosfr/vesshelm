use thiserror::Error;

#[derive(Error, Debug)]
pub enum ClientError {
    #[error(transparent)]
    Git(#[from] GitError),

    #[error(transparent)]
    Helm(#[from] HelmClientError),

    #[error(transparent)]
    Oci(#[from] OciError),

    #[error(transparent)]
    ArtifactHub(#[from] ArtifactHubError),
}

#[derive(Error, Debug)]
pub enum GitError {
    #[error("Failed to clone repository '{0}': {1}")]
    CloneFailed(String, #[source] git2::Error),

    #[error("Failed to checkout version '{0}': {1}")]
    CheckoutFailed(String, #[source] git2::Error),

    #[error("Reference '{0}' not found in repository")]
    RefNotFound(String),

    #[error("Git operation failed: {0}")]
    GitOpFailed(#[from] git2::Error),
}

#[derive(Error, Debug)]
pub enum HelmClientError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Failed to add repository '{0}': {1}")]
    RepoAddFailed(String, String),

    #[error("Failed to update repositories: {0}")]
    RepoUpdateFailed(String),

    #[error("Failed to pull chart '{0}': {1}")]
    PullFailed(String, String),

    #[error("Failed to list plugins: {0}")]
    PluginListFailed(String),

    #[error("Failed to install plugin '{0}': {1}")]
    PluginInstallFailed(String, String),

    #[error("Failed to show chart '{0}': {1}")]
    ShowChartFailed(String, String),

    #[error("Failed to uninstall release '{0}': {1}")]
    UninstallFailed(String, String),

    #[error("Helm search repo for '{0}' failed: {1}")]
    SearchRepoFailed(String, String),
}

#[derive(Error, Debug)]
pub enum ArtifactHubError {
    #[error("API query failed: {0}")]
    ApiQueryFailed(#[from] reqwest::Error),

    #[error("Package '{1}' not found in repository '{0}'")]
    PackageNotFound(String, String),

    #[error("API returned error status: {0}")]
    ApiError(reqwest::StatusCode),

    #[error("Failed to parse API response: {0}")]
    ResponseParseFailed(reqwest::Error),
}

#[derive(Error, Debug)]
pub enum OciError {
    #[error("Invalid OCI URL format")]
    InvalidUrlFormat,

    #[error("Registry requires auth but no Www-Authenticate header found")]
    MissingAuthHeader,

    #[error("Invalid Www-Authenticate header: {0}")]
    InvalidAuthHeader(String),

    #[error("Failed to fetch tags: {0}")]
    FetchTagsFailed(String),

    #[error("Token request failed: {0}")]
    TokenRequestFailed(String),

    #[error("Failed to connect to registry: {0}")]
    ConnectionFailed(String),

    #[error("Failed to parse response: {0}")]
    ParseFailed(String),

    #[error("Regex error: {0}")]
    RegexError(String),

    #[error("Field '{0}' not found in auth header")]
    FieldNotFound(String),

    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),
}
