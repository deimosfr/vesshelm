# CLI

## ADDED Requirements

### Requirement: Global Config Option
The tool MUST accept a global option to specify the configuration file path.

#### Scenario: Default Config Path
Given the user runs `vesshelm deploy` without any config argument
Then the tool MUST look for `vesshelm.yaml` in the current working directory.

#### Scenario: Custom Config Path
Given the user runs `vesshelm --config my-custom-config.yaml deploy`
Then the tool MUST use `my-custom-config.yaml` as the configuration file.
And it MUST NOT look for `vesshelm.yaml`.

#### Scenario: Missing Custom Config
Given the user runs `vesshelm --config non-existent.yaml deploy`
Then the tool MUST exit with an error indicating the configuration file was not found.

#### Scenario: Init with Config Path
Given the user runs `vesshelm --config new-config.yaml init`
Then the tool MUST create `new-config.yaml` instead of `vesshelm.yaml`.
