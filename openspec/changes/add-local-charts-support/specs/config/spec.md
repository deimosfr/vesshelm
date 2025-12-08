# Spec: config-local-charts

## MODIFIED Requirements

### Requirement: Chart Definitions
The system MUST support chart definitions where `repo_name` and `version` are optional.

#### Scenario: Local Chart Definition
- **WHEN** user defines a chart with `repo_name: null` and `version: null`.
- **THEN** configuration validation succeeds.
- **AND** `vesshelm sync` skips the chart.
- **AND** `vesshelm deploy` uses the configured path.
