use crate::clients::error::OciError;
use regex::Regex;
use reqwest::header::{AUTHORIZATION, WWW_AUTHENTICATE};
use reqwest::{Client, StatusCode};
use serde::Deserialize;
use std::time::Duration;

#[derive(Debug, Deserialize)]
struct TagList {
    #[allow(dead_code)]
    name: String,
    tags: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct TokenResponse {
    token: String,
    #[allow(dead_code)]
    #[serde(default)]
    access_token: Option<String>,
}

pub struct OciClient {
    client: Client,
}

impl Default for OciClient {
    fn default() -> Self {
        Self::new()
    }
}

impl OciClient {
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .user_agent(format!("vesshelm/{}", env!("CARGO_PKG_VERSION")))
            .build()
            .unwrap_or_default();
        Self { client }
    }

    pub async fn get_tags(
        &self,
        repo_url: &str,
        chart_name: &str,
    ) -> Result<Vec<String>, OciError> {
        let full_url = if let Some(stripped) = repo_url.strip_prefix("oci://") {
            stripped
        } else {
            repo_url
        };

        let (registry, path) = full_url.split_once('/').ok_or(OciError::InvalidUrlFormat)?;

        let path_suffix = format!("/{}", chart_name);
        let repo_path = if full_url.ends_with(&path_suffix) {
            path.to_string()
        } else {
            format!("{}/{}", path.trim_end_matches('/'), chart_name)
        };

        let tags_url = format!("https://{}/v2/{}/tags/list", registry, repo_path);

        // 1. Try request without auth
        let resp = self
            .client
            .get(&tags_url)
            .send()
            .await
            .map_err(|e| OciError::ConnectionFailed(e.to_string()))?;

        if resp.status().is_success() {
            let list: TagList = resp
                .json()
                .await
                .map_err(|e| OciError::ParseFailed(e.to_string()))?;
            return Ok(list.tags);
        }

        // 2. Handle 401 Unauthorized
        if resp.status() == StatusCode::UNAUTHORIZED {
            let auth_header = resp
                .headers()
                .get(WWW_AUTHENTICATE)
                .ok_or(OciError::MissingAuthHeader)?
                .to_str()
                .map_err(|e| OciError::InvalidAuthHeader(e.to_string()))?
                .to_string();

            let token = self.authenticate(&auth_header).await?;

            let resp = self
                .client
                .get(&tags_url)
                .header(AUTHORIZATION, format!("Bearer {}", token))
                .send()
                .await
                .map_err(|e| OciError::FetchTagsFailed(e.to_string()))?;

            if resp.status().is_success() {
                let list: TagList = resp
                    .json()
                    .await
                    .map_err(|e| OciError::ParseFailed(e.to_string()))?;
                return Ok(list.tags);
            } else {
                return Err(OciError::FetchTagsFailed(resp.status().to_string()));
            }
        }

        Err(OciError::FetchTagsFailed(resp.status().to_string()))
    }

    async fn authenticate(&self, auth_header: &str) -> Result<String, OciError> {
        let realm = self.extract_field(auth_header, "realm")?;
        let service = self.extract_field(auth_header, "service")?;
        let scope = self.extract_field(auth_header, "scope")?;

        let url = format!("{}?service={}&scope={}", realm, service, scope);

        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| OciError::TokenRequestFailed(e.to_string()))?;

        if !resp.status().is_success() {
            return Err(OciError::TokenRequestFailed(resp.status().to_string()));
        }

        let token_resp: TokenResponse = resp
            .json()
            .await
            .map_err(|e| OciError::ParseFailed(e.to_string()))?;

        Ok(token_resp.token)
    }

    fn extract_field(&self, header: &str, field: &str) -> Result<String, OciError> {
        let pattern = format!("{}=\"([^\"]+)\"", field);
        let re = Regex::new(&pattern).map_err(|e| OciError::RegexError(e.to_string()))?;

        re.captures(header)
            .and_then(|caps| caps.get(1))
            .map(|m| m.as_str().to_string())
            .ok_or_else(|| OciError::FieldNotFound(field.to_string()))
    }
}
