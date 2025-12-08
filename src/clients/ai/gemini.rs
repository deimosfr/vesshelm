use super::{AIClient, SRE_PROMPT};
use crate::config::AIConfig;
use crate::error::VesshelmError;
use async_trait::async_trait;
use reqwest::Client;
use serde_json::json;

pub struct GeminiClient {
    api_key: String,
    model: String,
    rtk_enabled: bool,
}

impl GeminiClient {
    pub fn new(config: &AIConfig) -> Self {
        Self {
            api_key: config.api_key.clone().unwrap_or_default(),
            model: config
                .model
                .clone()
                .unwrap_or_else(|| "gemini-1.5-flash".to_string()),
            rtk_enabled: config.rtk.diff_compression,
        }
    }
}

#[async_trait]
impl AIClient for GeminiClient {
    async fn analyze_upgrade(&self, diff: &str, overrides: &str) -> Result<String, VesshelmError> {
        let final_diff = if self.rtk_enabled {
            super::compress_diff(diff)
        } else {
            diff.to_string()
        };

        let client = Client::new();
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            self.model, self.api_key
        );

        let payload = json!({
            "contents": [{
                "parts": [
                    {"text": SRE_PROMPT},
                    {"text": format!("Git Diff:\n{}\n\nChart Overrides Context:\n{}", final_diff, overrides)}
                ]
            }]
        });

        let resp =
            client.post(url).json(&payload).send().await.map_err(|e| {
                VesshelmError::CommandFailed(format!("Gemini request failed: {}", e))
            })?;

        let status = resp.status();
        if !status.is_success() {
            let err_text = resp.text().await.unwrap_or_default();
            return Err(VesshelmError::CommandFailed(format!(
                "Gemini API error ({}): {}",
                status, err_text
            )));
        }

        let json: serde_json::Value = resp.json().await.map_err(|e| {
            VesshelmError::CommandFailed(format!("Failed to parse Gemini response: {}", e))
        })?;

        let text = json["candidates"][0]["content"]["parts"][0]["text"]
            .as_str()
            .ok_or_else(|| {
                VesshelmError::CommandFailed("Empty response from Gemini".to_string())
            })?;

        Ok(text.to_string())
    }
}
