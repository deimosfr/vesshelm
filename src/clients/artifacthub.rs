use anyhow::{Context, Result};
use reqwest::Client;
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct Package {
    pub name: String,
    pub version: String,
    pub repository: Repository,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Repository {
    pub name: String,
    pub url: String,
}

pub struct ArtifactHubClient {
    client: Client,
}

impl Default for ArtifactHubClient {
    fn default() -> Self {
        Self::new()
    }
}

impl ArtifactHubClient {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    pub async fn get_package_details(
        &self,
        repo_name: &str,
        package_name: &str,
    ) -> Result<Package> {
        let url = format!(
            "https://artifacthub.io/api/v1/packages/helm/{}/{}",
            repo_name, package_name
        );

        let resp = self
            .client
            .get(&url)
            .header("User-Agent", "vesshelm")
            .header("Accept", "application/json")
            .send()
            .await
            .context("Failed to query Artifact Hub API")?;

        if !resp.status().is_success() {
            if resp.status() == reqwest::StatusCode::NOT_FOUND {
                anyhow::bail!(
                    "Package {}/{} not found on Artifact Hub",
                    repo_name,
                    package_name
                );
            }
            anyhow::bail!("Artifact Hub API error: {}", resp.status());
        }

        let package: Package = resp
            .json()
            .await
            .context("Failed to parse Artifact Hub response")?;
        Ok(package)
    }
}
