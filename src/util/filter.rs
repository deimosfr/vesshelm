use std::collections::HashSet;

use anyhow::{Result, bail};

/// Validates that all charts specified in `only_args` exist in `available_names`.
///
/// # Arguments
/// * `available_names` - List of names of available charts.
/// * `only_args` - List of chart names specified by the user to filter.
///
/// # Returns
/// * `Ok(())` if all specified charts exist.
/// * `Err` if any specified chart is missing.
pub fn validate_only_args(available_names: &[&str], only_args: &[String]) -> Result<()> {
    if only_args.is_empty() {
        return Ok(());
    }

    let available_set: HashSet<_> = available_names.iter().cloned().collect();
    let missing: Vec<_> = only_args
        .iter()
        .filter(|name| !available_set.contains(name.as_str()))
        .collect();

    if !missing.is_empty() {
        let missing_list = missing
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        bail!(
            "The following charts specified in --only do not exist: {}",
            missing_list
        );
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_only_args_all_exist() {
        let charts = vec!["foo", "bar", "baz"];
        let only = vec!["foo".to_string(), "baz".to_string()];
        assert!(validate_only_args(&charts, &only).is_ok());
    }

    #[test]
    fn test_validate_only_args_missing_one() {
        let charts = vec!["foo", "bar"];
        let only = vec!["foo".to_string(), "missing".to_string()];
        let err = validate_only_args(&charts, &only).unwrap_err();
        assert_eq!(
            err.to_string(),
            "The following charts specified in --only do not exist: missing"
        );
    }

    #[test]
    fn test_validate_only_args_missing_multiple() {
        let charts = vec!["foo"];
        let only = vec!["missing1".to_string(), "missing2".to_string()];
        let err = validate_only_args(&charts, &only).unwrap_err();
        // The order of missing list depends on iteration, but since `only` is slice, it should be stable order of `only`
        assert_eq!(
            err.to_string(),
            "The following charts specified in --only do not exist: missing1, missing2"
        );
    }

    #[test]
    fn test_validate_only_args_empty_only_list() {
        let charts = vec!["foo"];
        let only = vec![];
        assert!(validate_only_args(&charts, &only).is_ok());
    }
}
