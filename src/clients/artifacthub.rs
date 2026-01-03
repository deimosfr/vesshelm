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
    base_url: String,
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
            base_url: "https://artifacthub.io/api/v1".to_string(),
        }
    }

    #[cfg(test)]
    pub fn with_base_url(base_url: &str) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.to_string(),
        }
    }

    pub async fn get_package_details(
        &self,
        repo_name: &str,
        package_name: &str,
    ) -> Result<Package> {
        let url = format!(
            "{}/packages/helm/{}/{}",
            self.base_url, repo_name, package_name
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::thread;

    #[tokio::test]
    async fn test_get_package_details_success() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let port = addr.port();

        thread::spawn(move || {
            let mut stream = listener.accept().unwrap().0;
            let mut buffer = [0; 1024];
            let _ = stream.read(&mut buffer).unwrap();

            let response_body = r#"{
                "name": "test-chart",
                "version": "1.2.3",
                "repository": {
                    "name": "test-repo",
                    "url": "https://charts.example.com"
                }
            }"#;

            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
                response_body.len(),
                response_body
            );

            stream.write_all(response.as_bytes()).unwrap();
            stream.flush().unwrap();
        });

        let base_url = format!("http://127.0.0.1:{}", port);
        let client = ArtifactHubClient::with_base_url(&base_url);

        let pkg = client
            .get_package_details("test-repo", "test-chart")
            .await
            .expect("Should succeed");

        assert_eq!(pkg.name, "test-chart");
        assert_eq!(pkg.version, "1.2.3");
        assert_eq!(pkg.repository.url, "https://charts.example.com");
    }

    #[tokio::test]
    async fn test_get_package_details_not_found() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let port = addr.port();

        thread::spawn(move || {
            let mut stream = listener.accept().unwrap().0;
            let mut buffer = [0; 512];
            let _ = stream.read(&mut buffer).unwrap();

            let response = "HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\n\r\n";
            stream.write_all(response.as_bytes()).unwrap();
        });

        let base_url = format!("http://127.0.0.1:{}", port);
        let client = ArtifactHubClient::with_base_url(&base_url);

        let result = client.get_package_details("repo", "missing").await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("not found"));
    }
}
