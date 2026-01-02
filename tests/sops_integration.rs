use anyhow::Result;
use std::path::Path;
use vesshelm::util::variables::load_variables;

#[test]
fn test_sops_decryption_attempt() -> Result<()> {
    let base_path = Path::new("tests/fixtures");
    let paths = vec!["dummy_sops.yaml".to_string()];

    let result = load_variables(&paths, base_path);

    // We expect an error because sops is likely not configured/installed or the file is invalid sops
    assert!(result.is_err());
    let err = result.unwrap_err();
    let err_msg = err.to_string();

    // Check if the error message comes from our code wrapper or sops
    // If detection worked, it TRIED to run sops.
    // If sops is missing: "SOPS encrypted file detected ..., but 'sops' binary is not found"
    // If sops is present but fails: "Failed to decrypt file ...: <sops stderr>"

    // So if the error confirms detection, then we are good.
    if err_msg.contains("SOPS encrypted file detected")
        || err_msg.contains("Failed to decrypt file")
    {
        println!(
            "Successfully detected SOPS file and attempted decryption: {}",
            err_msg
        );
        return Ok(());
    }

    // If it failed with "Failed to parse variable file" without mentioning decryption, then detection FAILED.
    println!("Unexpected error: {}", err_msg);
    panic!("Did not detect SOPS file correctly or failed in an unexpected way");
}
