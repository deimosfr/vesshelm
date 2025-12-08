# Validation Specs

## ADDED Requirements

### Requirement: Validations MUST identify the specific chart and repository when a repository is missing

When a chart references a repository that is not defined in the configuration, the validation error message MUST explicitly name both the chart and the missing repository to facilitate debugging.

#### Scenario: Missing repository in chart config

- **Given** a `vesshelm.yaml` with a chart named `nginx` referencing `missing-repo`
- **When** the user runs `vesshelm validate`
- **Then** the error output contains "Chart 'nginx' references a repository 'missing-repo' that does not exist"
