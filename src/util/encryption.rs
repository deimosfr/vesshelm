use anyhow::{Context, Result, anyhow};
use std::path::Path;
use std::process::Command;

/// Checks if the content appears to be a SOPS-encrypted file.
/// We look for "sops" and "mac" keys which are standard in SOPS files.
pub fn is_sops_encrypted(content: &str) -> bool {
    // Basic heuristic: check for sops metadata keys.
    // robust sops files usually have a root "sops" key containing "mac", "version", etc.
    // For a quick check without parsing full YAML (which might fail if encrypted values are invalid yaml strings, though SOPS usually keeps valid YAML structure),
    // we can check for substring existence.
    // "sops:" is the root key, "mac:" is the message authentication code.
    content.contains("sops:") && content.contains("mac:")
}

/// Decrypts a file using the `sops` binary.
/// Returns the decrypted content as a String.
pub fn decrypt_sops_file(path: &Path) -> Result<String> {
    // Check if sops is installed
    let version_check = Command::new("sops").arg("--version").output();

    if version_check.is_err() {
        return Err(anyhow!(
            "SOPS encrypted file detected ({:?}), but 'sops' binary is not found in PATH.\nPlease install sops to use encrypted variables.",
            path
        ));
    }

    let output = Command::new("sops")
        .arg("--decrypt")
        .arg("--output-type")
        .arg("yaml")
        .arg(path)
        .output()
        .context("Failed to execute sops command")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("Failed to decrypt file {:?}: {}", path, stderr));
    }

    let decrypted_content =
        String::from_utf8(output.stdout).context("Failed to parse sops output as UTF-8")?;

    Ok(decrypted_content)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_sops_encrypted() {
        let sops_content = r#"
key: ENC[AES256_GCM,data:...]
sops:
    mac: ENC[AES256_GCM,data:...]
    version: 3.7.1
"#;
        assert!(is_sops_encrypted(sops_content));

        let plain_content = r#"
key: value
sops_no_mac:
    version: 1.0
"#;
        assert!(!is_sops_encrypted(plain_content));
    }
}
