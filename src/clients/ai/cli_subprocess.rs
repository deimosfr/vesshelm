use crate::error::VesshelmError;
use std::process::Stdio;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

pub const DEBUG_ENV: &str = "VESSHELM_AI_DEBUG";

fn debug_enabled() -> bool {
    std::env::var(DEBUG_ENV)
        .map(|v| v != "0" && !v.is_empty())
        .unwrap_or(false)
}

pub enum PromptDelivery<'a> {
    /// Pipe the prompt to the child's stdin. Use when the CLI reads from stdin.
    Stdin,
    /// Append the prompt as a positional argument after the given flag, e.g. `-p`.
    /// The CLI is invoked as `binary <args...> <flag> <prompt>`.
    AsArg(&'a str),
}

pub async fn run_cli(
    binary: &str,
    args: &[&str],
    prompt: &str,
    delivery: PromptDelivery<'_>,
) -> Result<String, VesshelmError> {
    let debug = debug_enabled();

    let mut full_args: Vec<String> = args.iter().map(|s| s.to_string()).collect();
    if let PromptDelivery::AsArg(flag) = &delivery {
        full_args.push(flag.to_string());
        full_args.push(prompt.to_string());
    }

    if debug {
        let displayed_args: Vec<String> = match &delivery {
            PromptDelivery::AsArg(flag) => {
                let mut a = args.iter().map(|s| s.to_string()).collect::<Vec<_>>();
                a.push((*flag).to_string());
                a.push(format!("<prompt:{}B>", prompt.len()));
                a
            }
            PromptDelivery::Stdin => args.iter().map(|s| s.to_string()).collect(),
        };
        eprintln!("[ai-debug] spawn: {} {}", binary, displayed_args.join(" "));
        eprintln!(
            "[ai-debug] prompt: {} bytes ({} lines), delivery: {}",
            prompt.len(),
            prompt.lines().count(),
            match delivery {
                PromptDelivery::Stdin => "stdin",
                PromptDelivery::AsArg(_) => "arg",
            }
        );
        eprintln!("[ai-debug] --- begin prompt ---");
        eprintln!("{}", prompt);
        eprintln!("[ai-debug] --- end prompt ---");
        eprintln!("[ai-debug] streaming child stderr below:");
    }

    let stderr_mode = if debug {
        Stdio::inherit()
    } else {
        Stdio::piped()
    };
    let stdin_mode = match delivery {
        PromptDelivery::Stdin => Stdio::piped(),
        PromptDelivery::AsArg(_) => Stdio::null(),
    };

    let mut child = Command::new(binary)
        .args(&full_args)
        .stdin(stdin_mode)
        .stdout(Stdio::piped())
        .stderr(stderr_mode)
        .spawn()
        .map_err(|e| match e.kind() {
            std::io::ErrorKind::NotFound => VesshelmError::MissingDependency(format!(
                "AI CLI binary '{}' not found in PATH",
                binary
            )),
            _ => VesshelmError::CommandFailed(format!("spawn '{}' failed: {}", binary, e)),
        })?;

    if matches!(delivery, PromptDelivery::Stdin)
        && let Some(mut stdin) = child.stdin.take()
    {
        stdin
            .write_all(prompt.as_bytes())
            .await
            .map_err(|e| VesshelmError::CommandFailed(format!("write stdin: {}", e)))?;
        stdin.shutdown().await.ok();
    }

    let out = child
        .wait_with_output()
        .await
        .map_err(|e| VesshelmError::CommandFailed(format!("wait '{}': {}", binary, e)))?;

    if debug {
        eprintln!(
            "[ai-debug] child exited: {} (stdout={} bytes)",
            out.status,
            out.stdout.len()
        );
    }

    if !out.status.success() {
        let stderr_msg = if debug {
            "(streamed above)".to_string()
        } else {
            String::from_utf8_lossy(&out.stderr).trim().to_string()
        };
        let stdout_msg = String::from_utf8_lossy(&out.stdout);
        let stdout_tail = stdout_msg.trim();
        return Err(VesshelmError::CommandFailed(format!(
            "{} exited with {}: stderr={} | stdout={}",
            binary,
            out.status,
            if stderr_msg.is_empty() {
                "(empty)"
            } else {
                &stderr_msg
            },
            if stdout_tail.is_empty() {
                "(empty)"
            } else {
                stdout_tail
            }
        )));
    }
    Ok(strip_markdown_fences(&String::from_utf8_lossy(&out.stdout)))
}

pub fn build_prompt(diff: &str, overrides: &str) -> String {
    format!(
        "{}\n\nGit Diff:\n{}\n\nChart Overrides Context:\n{}",
        super::SRE_PROMPT,
        diff,
        overrides
    )
}

/// Strip leading/trailing markdown code fences from AI CLI output.
///
/// Some AI CLIs wrap their response in ``` fences despite prompt instructions.
/// Only strips when both opening and closing fences are present so legitimate
/// inline code blocks in the middle of a response are preserved.
fn strip_markdown_fences(s: &str) -> String {
    let trimmed = s.trim_end_matches(['\n', '\r', ' ', '\t']);
    let lines: Vec<&str> = trimmed.lines().collect();

    let first_idx = lines.iter().position(|l| !l.trim().is_empty());
    let last_idx = lines.iter().rposition(|l| !l.trim().is_empty());

    let (Some(first), Some(last)) = (first_idx, last_idx) else {
        return s.to_string();
    };

    if first == last {
        return s.to_string();
    }

    let opens = lines[first].trim_start().starts_with("```");
    let closes = lines[last].trim() == "```";

    if !(opens && closes) {
        return s.to_string();
    }

    let inner = &lines[first + 1..last];
    let mut out = inner.join("\n");
    if !out.is_empty() {
        out.push('\n');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::strip_markdown_fences;

    #[test]
    fn strips_plain_fences() {
        let input = "```\n[OK] foo\n[WARN] bar\n```\n";
        assert_eq!(strip_markdown_fences(input), "[OK] foo\n[WARN] bar\n");
    }

    #[test]
    fn strips_fences_with_lang_tag() {
        let input = "```text\nhello\nworld\n```";
        assert_eq!(strip_markdown_fences(input), "hello\nworld\n");
    }

    #[test]
    fn passthrough_when_no_fences() {
        let input = "[OK] all good\n";
        assert_eq!(strip_markdown_fences(input), input);
    }

    #[test]
    fn passthrough_when_only_opening_fence() {
        let input = "```\nstuff\nno close\n";
        assert_eq!(strip_markdown_fences(input), input);
    }

    #[test]
    fn ignores_leading_and_trailing_blank_lines() {
        let input = "\n\n```\nbody\n```\n\n";
        assert_eq!(strip_markdown_fences(input), "body\n");
    }
}
