# Spec: CLI Initialization

## ADDED Requirements

### Requirement: Init Diff Defaults
When initializing a new configuration, Vesshelm MUST include default helm diff settings.

#### Scenario: Init Generates Diff Config
Given a directory without `vesshelm.yaml`,
When `vesshelm init` is executed,
Then the created `vesshelm.yaml` should contain `diff_enabled: true`.
