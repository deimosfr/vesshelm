use super::{AIClient, SRE_PROMPT};
use crate::config::AIConfig;
use crate::error::VesshelmError;
use async_trait::async_trait;
use reqwest::Client;
use serde_json::json;

pub struct ClaudeClient {
    api_key: String,
    model: String,
    rtk_enabled: bool,
}

impl ClaudeClient {
    pub fn new(config: &AIConfig) -> Self {
        Self {
            api_key: config.api_key.clone().unwrap_or_default(),
            model: config
                .model
                .clone()
                .unwrap_or_else(|| "claude-3-5-sonnet-20240620".to_string()),
            rtk_enabled: config.rtk.diff_compression,
        }
    }
}

#[async_trait]
impl AIClient for ClaudeClient {
    async fn analyze_upgrade(&self, diff: &str, overrides: &str) -> Result<String, VesshelmError> {
        let final_diff = if self.rtk_enabled {
            super::compress_diff(diff)
        } else {
            diff.to_string()
        };

        let client = Client::new();
        let url = "https://api.anthropic.com/v1/messages";

        let payload = json!({
            "model": self.model,
            "max_tokens": 4096,
            "system": SRE_PROMPT,
            "messages": [
                {"role": "user", "content": format!("Git Diff:\n{}\n\nChart Overrides Context:\n{}", final_diff, overrides)}
            ]
        });

        let resp = client
            .post(url)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&payload)
            .send()
            .await
            .map_err(|e| VesshelmError::CommandFailed(format!("Claude request failed: {}", e)))?;

        let status = resp.status();
        if !status.is_success() {
            let err_text = resp.text().await.unwrap_or_default();
            return Err(VesshelmError::CommandFailed(format!(
                "Claude API error ({}): {}",
                status, err_text
            )));
        }

        let json: serde_json::Value = resp.json().await.map_err(|e| {
            VesshelmError::CommandFailed(format!("Failed to parse Claude response: {}", e))
        })?;

        let text = json["content"][0]["text"].as_str().ok_or_else(|| {
            VesshelmError::CommandFailed("Empty response from Claude".to_string())
        })?;

        Ok(text.to_string())
    }
}
