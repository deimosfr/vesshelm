# Configuration Validation

## ADDED Requirements

### Validate Values Files
The configuration validator ensures that all referenced files exist.

#### Requirement: File existence check
The validator must verify that `values_files` paths point to existing files.

#### Scenario: Fails when values file is missing
Given a `vesshelm.yaml` configuration
And a chart defines `values_files` with a path that does not exist
When I run `vesshelm validate`
Then it should fail with an error indicating the missing file
