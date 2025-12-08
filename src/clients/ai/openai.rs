use super::{AIClient, SRE_PROMPT};
use crate::config::AIConfig;
use crate::error::VesshelmError;
use async_trait::async_trait;
use reqwest::Client;
use serde_json::json;

pub struct OpenAIClient {
    api_key: String,
    model: String,
    rtk_enabled: bool,
}

impl OpenAIClient {
    pub fn new(config: &AIConfig) -> Self {
        Self {
            api_key: config.api_key.clone().unwrap_or_default(),
            model: config.model.clone().unwrap_or_else(|| "gpt-4o".to_string()),
            rtk_enabled: config.rtk.diff_compression,
        }
    }
}

#[async_trait]
impl AIClient for OpenAIClient {
    async fn analyze_upgrade(&self, diff: &str, overrides: &str) -> Result<String, VesshelmError> {
        let final_diff = if self.rtk_enabled {
            super::compress_diff(diff)
        } else {
            diff.to_string()
        };

        let client = Client::new();
        let url = "https://api.openai.com/v1/chat/completions";

        let payload = json!({
            "model": self.model,
            "messages": [
                {"role": "system", "content": SRE_PROMPT},
                {"role": "user", "content": format!("Git Diff:\n{}\n\nChart Overrides Context:\n{}", final_diff, overrides)}
            ]
        });

        let resp = client
            .post(url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&payload)
            .send()
            .await
            .map_err(|e| VesshelmError::CommandFailed(format!("OpenAI request failed: {}", e)))?;

        let status = resp.status();
        if !status.is_success() {
            let err_text = resp.text().await.unwrap_or_default();
            return Err(VesshelmError::CommandFailed(format!(
                "OpenAI API error ({}): {}",
                status, err_text
            )));
        }

        let json: serde_json::Value = resp.json().await.map_err(|e| {
            VesshelmError::CommandFailed(format!("Failed to parse OpenAI response: {}", e))
        })?;

        let text = json["choices"][0]["message"]["content"]
            .as_str()
            .ok_or_else(|| {
                VesshelmError::CommandFailed("Empty response from OpenAI".to_string())
            })?;

        Ok(text.to_string())
    }
}
