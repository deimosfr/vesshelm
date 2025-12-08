# Spec Delta: Enable Helm Diff Colors

## MODIFIED Requirements

### Deployment Diff
The `deploy` command SHOULD display a unified diff of changes before deployment.

#### Scenario: Colorized Output
When `helm diff` is executed, it MUST be configured to force colored output, ensuring readability even when output is piped.
- `HELM_DIFF_COLOR` environment variable MUST be set to `true`.
