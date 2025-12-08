## ADDED Requirements

### Requirement: secrets_files configuration field
The system MUST support a `secrets_files` field in the configuration file, which accepts a list of file paths.
#### Scenario: Parse configuration with secrets_files
Given a `vesshelm.yaml` with a `secrets_files` list
When the configuration is loaded
Then the `secrets_files` field should contain the specified paths

### Requirement: secrets_files validation
The system MUST validate that files listed in `secrets_files` exist on the filesystem.
#### Scenario: Fail validation if secrets file is missing
Given `vesshelm.yaml` refers to a non-existent file in `secrets_files`
When the configuration is validated
Then it should return a validation error indicating the missing file

### Requirement: Load and merge secrets
The system MUST load files from `secrets_files` and merge them into the variable context, allowing them to participate in value interpolation.
#### Scenario: Interpolate value from secrets file
Given a secret defined in a file listed in `secrets_files`
When a chart argument references this secret
Then the argument should be interpolated with the secret value

### Requirement: Variable and Secret Interpolation
The system MUST render the content of each file in `variable_files` and `secrets_files` as a template using the cumulative context of previously loaded variables.
#### Scenario: Recursive interpolation
Given `vars1.yaml` defines `env: prod`
And `vars2.yaml` defines `url: "http://{{ env }}.com"`
When variables are loaded
Then `url` should resolve to `"http://prod.com"`

### Requirement: secret_files Alias
The system MUST support `secret_files` (singular) as an alias for `secrets_files` in the configuration file to support backward compatibility or user preference.
#### Scenario: Parse configuration with secret_files
Given a `vesshelm.yaml` using `secret_files` instead of `secrets_files`
When the configuration is loaded
Then the system should treat it exactly as `secrets_files`
