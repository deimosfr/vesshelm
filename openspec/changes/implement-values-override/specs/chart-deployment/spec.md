# Spec: chart-deployment

## ADDED Requirements

### Requirement: Values File Support
The system MUST allow users to specify external values files for a chart.

#### Scenario: Apply Values Files
- **WHEN** Chart config has `values_files` list.
- **THEN** Deployment command includes `-f <path>` for each file.

### Requirement: Inline Values Support
The system MUST allow users to specify inline values in the configuration.

#### Scenario: Apply Inline Values
- **WHEN** Chart config has `values` list.
- **THEN** System converts these values to a format Helm accepts (e.g., temporary values file) and applies them.
