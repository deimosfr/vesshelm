## ADDED Requirements

### Requirement: Validation Error Specificity
Configuration validation errors MUST provide specific details about the failure, including relevant file paths or values.

#### Scenario: Missing variable file
Given I have a `vesshelm.yaml` with `variables_files` containing a non-existent file path "/path/to/missing.yaml"
When I run `vesshelm deploy` (or any command loading config)
Then the error message should contain "variable_file_not_found"
And the error message should contain "/path/to/missing.yaml"

#### Scenario: Missing secret file
Given I have a `vesshelm.yaml` with `secrets_files` containing a non-existent file path "/path/to/missing-secret.yaml"
When I run `vesshelm deploy`
Then the error message should contain "secrets_file_not_found"
And the error message should contain "/path/to/missing-secret.yaml"
