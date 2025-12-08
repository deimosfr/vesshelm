use crate::config::{AIConfig, AIMode, AIProvider as Provider};
use crate::error::VesshelmError;
use async_trait::async_trait;

pub mod claude;
pub mod claude_cli;
pub mod cli_subprocess;
pub mod codex_cli;
pub mod gemini;
pub mod gemini_cli;
pub mod ollama;
pub mod openai;

#[async_trait]
pub trait AIClient: Send + Sync {
    async fn analyze_upgrade(&self, diff: &str, overrides: &str) -> Result<String, VesshelmError>;
}

pub fn create_client(config: &AIConfig) -> Result<Box<dyn AIClient>, VesshelmError> {
    match (&config.provider, &config.mode) {
        (Provider::Gemini, AIMode::Api) => Ok(Box::new(gemini::GeminiClient::new(config))),
        (Provider::Claude, AIMode::Api) => Ok(Box::new(claude::ClaudeClient::new(config))),
        (Provider::OpenAI, AIMode::Api) => Ok(Box::new(openai::OpenAIClient::new(config))),
        (Provider::Ollama, _) => Ok(Box::new(ollama::OllamaClient::new(config))),
        (Provider::Claude, AIMode::Cli) => Ok(Box::new(claude_cli::ClaudeCliClient::new(config))),
        (Provider::Gemini, AIMode::Cli) => Ok(Box::new(gemini_cli::GeminiCliClient::new(config))),
        (Provider::OpenAI, AIMode::Cli) => Ok(Box::new(codex_cli::CodexCliClient::new(config))),
    }
}

pub const SRE_PROMPT: &str = r#"
You are a Senior Site Reliability Engineer (SRE).
Analyze the provided Git diff of Helm chart updates and the technical overrides for these charts.

Your goal is to provide a concise, plain-text technical report for a terminal.

CORE INSTRUCTIONS:
1. FOCUS ON OVERRIDES: Cross-reference every templating/values change in the diff against the "Chart Overrides Context". Only flag impacts that are confirmed by reading the provided overrides.
2. IDENTIFY BREAKAGE: Use the diff and your knowledge of applications (changelogs) to detect conflicts with these overrides.
3. GENERAL CHANGES: After analyzing overrides, mention critical changes in the chart's default templates or values that might impact the deployment.
4. NO HEDGING ON OVERRIDES: The "Chart Overrides Context" contains the FULL content of the user's values, values files, and helm args. Inspect it directly. Do NOT use conditional language like "if your override defines X", "if your file contains Y", or "you may need to check". Either the override defines the key (state it definitively, cite the override file path) or it does not (omit the bullet entirely). Treat absent keys as not-overridden; do not warn about them.
5. NO MARKDOWN: Do not use #, *, _, or any markdown formatting. Use plain text only.
6. NO DIFF: Do not include raw diff snippets in your response.
7. STATUS TAGS: Start each chart analysis with exactly one tag:
   [OK]    No override conflict and no behavior change for the current deployment.
           Use this even when general changes exist, as long as nothing breaks
           and no override is invalidated. Latent or future risks (e.g. "if you
           scale X up later", "if you enable Y in the future") MUST stay [OK];
           mention them under OVERRIDE_IMPACT but do not escalate the tag.
   [WARN]  At least one finding changes behavior of the current deployment,
           invalidates an existing override, or requires a config action to keep
           working after upgrade.
   [ERROR] Upgrade will fail or break the running deployment as currently
           configured (CRD removal, required field missing, incompatible value).
8. NO GENERIC ADVICE: Do not mention release notes, changelogs, or testing in staging.
9. IGNORE DEFAULT IMAGE BUMPS: Do not report on standard image tag or digest changes in the chart defaults if the user is not overriding them.
10. PINNED IMAGES: If the chart's default image version has been updated (check the diff for changes in chart values), but the user's configuration pins a specific image version, you MUST warn the user that their override is "sticking" the application to the old version, preventing the upgrade from taking effect.

FORMATTING RULES:
- Use indentation (4 spaces) for the description under the chart header.
- Use bullet points (*) for individual findings.
- Use EXACT markers for sections: "OVERRIDE_IMPACT:" and "GENERAL_CHANGE:".

Example Output (active override break, tag is [WARN]):
[WARN] alloy (monitoring)
    OVERRIDE_IMPACT:
    * Your override at helm/overrides/alloy.yaml sets 'alloy.config'. The new chart renames it to 'config.content', so this override is now ignored and the agent will start with default config after upgrade.
    GENERAL_CHANGE:
    * Default memory limit increased from 128Mi to 256Mi.

Example Output (only latent risk, tag stays [OK]):
[OK] kube-prometheus-stack (observability)
    OVERRIDE_IMPACT:
    * helm/overrides/kube-prometheus-stack.yaml sets grafana.replicas: 0. Grafana sub-chart bumped to 13.x which requires an image renderer token when imageRenderer is active. Pod never starts at replicas 0, so no impact today; revisit if you scale grafana up later.
    GENERAL_CHANGE:
    * Prometheus image bumped from v3.10.0 to v3.11.2.
"#;

pub fn compress_diff(diff: &str) -> String {
    let mut compressed = String::new();
    let lines = diff.lines();

    for line in lines {
        // Skip common meta lines that don't add value to SRE review of content
        if line.starts_with("index ") || line.starts_with("diff --git ") {
            continue;
        }

        // Keep file headers and content
        compressed.push_str(line);
        compressed.push('\n');
    }

    compressed
}
