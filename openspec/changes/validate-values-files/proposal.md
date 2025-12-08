# Validate Values Files

## Summary
Update the configuration validation to check that all file paths specified in `values_files` actually exist on the filesystem.

## Motivation
Currently, if a user specifies a non-existent `values_files` path in `vesshelm.yaml`, the validation passes. The tool likely fails later when trying to use that file (e.g., in helm command or reading it). Early validation provides immediate feedback to the user.

## Proposed Solution
Modify `src/config.rs` within the `validate_config` function to iterate over all `values_files` in each chart.
Check if each file exists using `std::path::Path::new(file).exists()`.
If a file is missing, return a validation error `values_file_not_found`.
