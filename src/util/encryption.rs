use crate::util::error::EncryptionError;
use std::path::Path;

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
/// Uses a 30 second timeout to prevent hanging.
pub async fn decrypt_sops_file(path: &Path) -> Result<String, EncryptionError> {
    let child = tokio::process::Command::new("sops")
        .arg("--decrypt")
        .arg("--output-type")
        .arg("yaml")
        .arg(path)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .kill_on_drop(true)
        .spawn();

    let child = match child {
        Ok(c) => c,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return Err(EncryptionError::SopsNotFound);
        }
        Err(e) => return Err(EncryptionError::SopsExecutionFailed(e)),
    };

    let output =
        match tokio::time::timeout(std::time::Duration::from_secs(30), child.wait_with_output())
            .await
        {
            Ok(Ok(output)) => output,
            Ok(Err(e)) => return Err(EncryptionError::SopsExecutionFailed(e)),
            Err(_) => {
                // Timeout occurred
                return Err(EncryptionError::DecryptionFailed(
                    path.to_string_lossy().to_string(),
                    "Operation timed out after 30 seconds".to_string(),
                ));
            }
        };

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(EncryptionError::DecryptionFailed(
            path.to_string_lossy().to_string(),
            stderr.to_string(),
        ));
    }

    let decrypted_content = String::from_utf8(output.stdout).map_err(EncryptionError::Utf8Error)?;

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
