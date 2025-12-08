#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;
    use vesshelm::config::Config;
    use vesshelm::util::error::format_error;

    #[test]
    fn test_parse_error_includes_filename() {
        let dir = tempdir().unwrap();
        let config_path = dir.path().join("vesshelm.yaml");
        let mut file = File::create(&config_path).unwrap();
        writeln!(file, "invalid: yaml: content: [").unwrap(); // Malformed YAML

        let result = Config::load_from_path(&config_path);
        assert!(result.is_err());
        let err = result.err().unwrap();
        let msg = format_error(&err);

        println!("Error: {}", msg);
        if msg.contains("vesshelm.yaml") {
            println!("SUCCESS: filename found in parse error");
        } else {
            panic!("Filename NOT found in parse error: {}", msg);
        }
    }

    #[test]
    fn test_default_dest_not_found_includes_chart_name() {
        let dir = tempdir().unwrap();
        let config_path = dir.path().join("vesshelm.yaml");
        let mut file = File::create(&config_path).unwrap();
        writeln!(
            file,
            r#"
repositories: []
charts:
  - name: my-chart
    namespace: default
destinations: []
"#
        )
        .unwrap();

        let result = Config::load_from_path(&config_path);
        // Load might succeed, but resolving destination should fail
        assert!(result.is_ok());
        let config = result.unwrap();
        let chart = &config.charts[0];

        let dest_res = config.resolve_chart_destination(chart);
        assert!(dest_res.is_err());
        let err = dest_res.err().unwrap();
        let msg = format_error(&err);

        println!("Error: {}", msg);
        if msg.contains("my-chart") {
            println!("SUCCESS: chart name found in default dest error");
        } else {
            panic!("Chart name NOT found in default dest error: {}", msg);
        }
    }
}
