use anyhow::{Context, Result};
use dialoguer::{Input, theme::ColorfulTheme};
use regex::Regex;
use vesshelm::clients::artifacthub::{ArtifactHubClient, Package};
use vesshelm::config::RepoType;

pub struct ChartDetails {
    pub repo_name: String,
    pub repo_url: String,
    pub chart_name: String,
    pub version: Option<String>,
    pub chart_path: Option<String>,
    pub repo_type: RepoType,
    pub comment: Option<String>,
}

#[async_trait::async_trait]
pub trait ChartSource {
    async fn prompt_details(&self) -> Result<ChartDetails>;
}

pub struct ArtifactHubSource;
pub struct GitSource;
pub struct OciSource;

fn parse_ah_url(url: &str) -> Result<(String, String)> {
    let re = Regex::new(r"artifacthub\.io/packages/helm/([^/]+)/([^/?]+)")?;
    let caps = re.captures(url).ok_or_else(|| anyhow::anyhow!("Invalid Artifact Hub URL format. Expected: https://artifacthub.io/packages/helm/<repo>/<chart>"))?;
    let repo = caps
        .get(1)
        .map(|m| m.as_str().to_string())
        .unwrap_or_default();
    let chart = caps
        .get(2)
        .map(|m| m.as_str().to_string())
        .unwrap_or_default();
    Ok((repo, chart))
}

fn derive_git(url: &str, chart_path: &str) -> (String, String) {
    let repo_name = url
        .trim_end_matches('/')
        .split('/')
        .next_back()
        .unwrap_or("git-repo")
        .to_string();
    let chart_name = chart_path
        .trim_end_matches('/')
        .split('/')
        .next_back()
        .unwrap_or("chart")
        .to_string();
    (repo_name, chart_name)
}

fn derive_oci(url: &str) -> (String, String, String) {
    let parts: Vec<&str> = url.split('/').collect();
    let (repo_url, chart_name_derived) = if let Some((last, rest)) = parts.split_last() {
        (rest.join("/"), last.to_string())
    } else {
        (url.to_string(), "oci-chart".to_string())
    };
    // For OCI repo name derivation, we use chart name as default?
    let repo_name = chart_name_derived.clone();
    (repo_name, repo_url, chart_name_derived)
}

fn map_package_to_details(package: Package, comment: Option<String>) -> ChartDetails {
    let repo_type = if package.repository.url.starts_with("oci://") {
        RepoType::Oci
    } else {
        RepoType::Helm
    };

    ChartDetails {
        repo_name: package.name.clone(),
        repo_url: package.repository.url,
        chart_name: package.name,
        version: Some(package.version),
        chart_path: None,
        repo_type,
        comment,
    }
}

#[async_trait::async_trait]
impl ChartSource for ArtifactHubSource {
    async fn prompt_details(&self) -> Result<ChartDetails> {
        let url: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Enter Artifact Hub URL")
            .interact_text()
            .context("Failed to read URL input")?;

        let (repo_arg, chart_arg) = parse_ah_url(&url)?;
        println!("Detected: Repo={}, Chart={}", repo_arg, chart_arg);

        let client = ArtifactHubClient::new();
        let package: Package = client
            .get_package_details(&repo_arg, &chart_arg)
            .await
            .context("Failed to fetch package details")?;

        println!(
            "Found chart: {} v{} from repo {}",
            package.name, package.version, package.repository.url
        );

        Ok(map_package_to_details(package, Some(url)))
    }
}

#[async_trait::async_trait]
impl ChartSource for GitSource {
    async fn prompt_details(&self) -> Result<ChartDetails> {
        let url: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Enter Git Repository URL")
            .interact_text()?;

        let chart_path: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Enter path to chart in repo")
            .interact_text()?;

        let version: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Enter Version (commit/tag/branch)")
            .interact_text()?;

        let (repo_name, chart_name) = derive_git(&url, &chart_path);

        Ok(ChartDetails {
            repo_name,
            repo_url: url.clone(),
            chart_name,
            version: Some(version),
            chart_path: Some(chart_path),
            repo_type: RepoType::Git,
            comment: Some(url),
        })
    }
}

#[async_trait::async_trait]
impl ChartSource for OciSource {
    async fn prompt_details(&self) -> Result<ChartDetails> {
        let url: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Enter OCI URL")
            .interact_text()?;

        let version: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Enter Version")
            .interact_text()?;

        let (repo_name, repo_url, chart_name) = derive_oci(&url);

        Ok(ChartDetails {
            repo_name,
            repo_url,
            chart_name,
            version: Some(version),
            chart_path: None,
            repo_type: RepoType::Oci,
            comment: Some(url),
        })
    }
}

pub fn get_source(selection: usize) -> Option<Box<dyn ChartSource>> {
    match selection {
        0 => Some(Box::new(ArtifactHubSource)),
        1 => Some(Box::new(GitSource)),
        2 => Some(Box::new(OciSource)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use vesshelm::clients::artifacthub::Repository;

    use super::*;

    #[test]
    fn test_parse_ah_url_valid() {
        let url = "https://artifacthub.io/packages/helm/gissilabs/vaultwarden";
        let (repo, chart) = parse_ah_url(url).unwrap();
        assert_eq!(repo, "gissilabs");
        assert_eq!(chart, "vaultwarden");
    }

    #[test]
    fn test_parse_ah_url_invalid() {
        let url = "https://google.com";
        assert!(parse_ah_url(url).is_err());
    }

    #[test]
    fn test_derive_git() {
        let url = "https://github.com/kubernetes-csi/csi-driver-smb";
        let path = "charts/v1.19.1/csi-driver-smb";
        let (repo, chart) = derive_git(url, path);
        assert_eq!(repo, "csi-driver-smb");
        assert_eq!(chart, "csi-driver-smb");
    }

    #[test]
    fn test_map_package_helm() {
        let pkg = Package {
            name: "chart-helm".to_string(),
            version: "1.2.3".to_string(),
            repository: Repository {
                name: "repo".to_string(),
                url: "https://charts.example.com".to_string(),
            },
        };

        let details = map_package_to_details(pkg, None);
        assert_eq!(details.repo_type, RepoType::Helm);
        assert_eq!(details.repo_url, "https://charts.example.com");
    }

    #[test]
    fn test_map_package_oci() {
        let pkg = Package {
            name: "chart-oci".to_string(),
            version: "0.0.1".to_string(),
            repository: Repository {
                name: "repo".to_string(),
                url: "oci://registry.example.com/repo/chart-oci".to_string(),
            },
        };

        let details = map_package_to_details(pkg, None);
        assert_eq!(details.repo_type, RepoType::Oci);
        assert_eq!(
            details.repo_url,
            "oci://registry.example.com/repo/chart-oci"
        );
    }

    #[test]
    fn test_derive_oci() {
        let url = "oci://code.forgejo.org/forgejo-helm/forgejo";
        let (repo_name, repo_url, chart_name) = derive_oci(url);
        assert_eq!(repo_name, "forgejo");
        assert_eq!(chart_name, "forgejo");
        assert_eq!(repo_url, "oci://code.forgejo.org/forgejo-helm");
    }
}
