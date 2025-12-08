#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;
    use vesshelm::config::Config;
    use vesshelm::util::error::format_error;

    #[test]
    fn test_variable_file_not_found_error_message() {
        // Create a temporary directory
        let dir = tempdir().unwrap();
        let config_path = dir.path().join("vesshelm.yaml");

        // write config with missing variable file
        let mut file = File::create(&config_path).unwrap();
        writeln!(
            file,
            r#"
variables_files:
  - /path/to/missing_variable_file.yaml
secrets_files:
  - /path/to/missing_secret_file.yaml

repositories: []
charts: []
destinations: []
"#
        )
        .unwrap();

        // Load config and expect error
        let result = Config::load_from_path(&config_path);

        assert!(result.is_err());
        let err = result.err().unwrap();

        // Use the util::error::format_error which is what CLI uses
        let msg = format_error(&err);

        println!("Error message: {}", msg);

        // Check for parts of the message to match regardless of color codes
        if msg.contains("Variable file not found")
            && msg.contains("/path/to/missing_variable_file.yaml")
        {
            println!("SUCCESS: variable file path found in error message");
        } else {
            println!("FAILURE: variable file path NOT found in error message");
            panic!(
                "Validation error does not contain the expected message. Got: {}",
                msg
            );
        }
    }
}
