use super::{AIClient, SRE_PROMPT};
use crate::config::AIConfig;
use crate::error::VesshelmError;
use async_trait::async_trait;
use reqwest::Client;
use serde_json::json;

pub struct OllamaClient {
    endpoint: String,
    model: String,
    rtk_enabled: bool,
}

impl OllamaClient {
    pub fn new(config: &AIConfig) -> Self {
        Self {
            endpoint: config
                .endpoint
                .clone()
                .unwrap_or_else(|| "http://localhost:11434".to_string()),
            model: config.model.clone().unwrap_or_else(|| "llama3".to_string()),
            rtk_enabled: config.rtk.diff_compression,
        }
    }
}

#[async_trait]
impl AIClient for OllamaClient {
    async fn analyze_upgrade(&self, diff: &str, overrides: &str) -> Result<String, VesshelmError> {
        let final_diff = if self.rtk_enabled {
            super::compress_diff(diff)
        } else {
            diff.to_string()
        };

        let client = Client::new();
        let url = format!("{}/api/chat", self.endpoint);

        let payload = json!({
            "model": self.model,
            "stream": false,
            "messages": [
                {"role": "system", "content": SRE_PROMPT},
                {"role": "user", "content": format!("Git Diff:\n{}\n\nChart Overrides Context:\n{}", final_diff, overrides)}
            ]
        });

        let resp =
            client.post(url).json(&payload).send().await.map_err(|e| {
                VesshelmError::CommandFailed(format!("Ollama request failed: {}", e))
            })?;

        let status = resp.status();
        if !status.is_success() {
            let err_text = resp.text().await.unwrap_or_default();
            return Err(VesshelmError::CommandFailed(format!(
                "Ollama API error ({}): {}",
                status, err_text
            )));
        }

        let json: serde_json::Value = resp.json().await.map_err(|e| {
            VesshelmError::CommandFailed(format!("Failed to parse Ollama response: {}", e))
        })?;

        let text = json["message"]["content"].as_str().ok_or_else(|| {
            VesshelmError::CommandFailed("Empty response from Ollama".to_string())
        })?;

        Ok(text.to_string())
    }
}
