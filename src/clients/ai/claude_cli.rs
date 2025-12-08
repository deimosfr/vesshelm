use super::AIClient;
use crate::config::AIConfig;
use crate::error::VesshelmError;
use async_trait::async_trait;

pub struct ClaudeCliClient {
    binary: String,
    model: Option<String>,
    extra_args: Vec<String>,
    rtk_enabled: bool,
}

impl ClaudeCliClient {
    pub fn new(config: &AIConfig) -> Self {
        Self {
            binary: config
                .cli_binary
                .clone()
                .unwrap_or_else(|| "claude".to_string()),
            model: config.model.clone(),
            extra_args: config.cli_args.clone(),
            rtk_enabled: config.rtk.diff_compression,
        }
    }
}

#[async_trait]
impl AIClient for ClaudeCliClient {
    async fn analyze_upgrade(&self, diff: &str, overrides: &str) -> Result<String, VesshelmError> {
        let final_diff = if self.rtk_enabled {
            super::compress_diff(diff)
        } else {
            diff.to_string()
        };
        let prompt = super::cli_subprocess::build_prompt(&final_diff, overrides);

        // `--print` non-interactive, `--tools ""` disables all tools so the model
        // returns prose directly, `--permission-mode plan` for safety (read-only).
        // Note: `--bare` is intentionally NOT used because it disables OAuth/keychain
        // reads, breaking subscription auth ("Not logged in" errors).
        let mut args: Vec<&str> = vec![
            "--print",
            "--tools",
            "",
            "--permission-mode",
            "plan",
            "--input-format",
            "text",
            "--output-format",
            "text",
        ];
        if let Some(m) = &self.model {
            args.extend_from_slice(&["--model", m]);
        }
        args.extend(self.extra_args.iter().map(String::as_str));
        super::cli_subprocess::run_cli(
            &self.binary,
            &args,
            &prompt,
            super::cli_subprocess::PromptDelivery::Stdin,
        )
        .await
    }
}
