# Spec: cli-commands-filtering

## ADDED Requirements

### Requirement: Sync Filtering
The system MUST allow users to sync a subset of charts.

#### Scenario: Sync Single Chart
- **WHEN** User runs `vesshelm sync --only my-chart`.
- **THEN** Only `my-chart` is synced.

### Requirement: Deploy Filtering
The system MUST allow users to deploy a subset of charts.

#### Scenario: Deploy Single Chart
- **WHEN** User runs `vesshelm deploy --only my-chart`.
- **THEN** Only `my-chart` is deployed.

### Requirement: Validation
The system MUST validate that charts specified in `--only` exist in the configuration.
**Technical Constraint:** This validation logic SHOULD be implemented in a dedicated, testable function to ensure consistency between `sync` and `deploy` commands.

#### Scenario: Chart Not Found
- **GIVEN** `my-chart` does not exist in `vesshelm.yaml`.
- **WHEN** User runs `vesshelm sync --only my-chart`.
- **THEN** The command fails with an error message explicitly stating `my-chart` was not found.

#### Scenario: Multiple Charts Partial Failure
- **GIVEN** `chart-a` exists but `chart-b` does not.
- **WHEN** User runs `vesshelm deploy --only chart-a,chart-b`.
- **THEN** The command fails with an error message explicitly stating `chart-b` was not found.
