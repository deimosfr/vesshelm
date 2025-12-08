# Spec: Chart Deployment

## ADDED Requirements

### Requirement: dry-run deployment
The `deploy` command MUST support a `--dry-run` flag.

#### Scenario: Running Dry Run
Given a configured chart,
When `vesshelm deploy --dry-run` is executed,
Then it should execute `helm diff ...` instead of `helm upgrade ...`.

### Requirement: Interactive Deployment
The `deploy` command MUST be interactive by default, running a diff first and asking for confirmation.
- It MUST support a `--no-interactive` flag to bypass confirmation.
- It MUST automatically skip deployment if the diff is empty (no changes).

#### Scenario: Interactive Flow
Given `deploy` is run without flags,
And a diff exists,
When the diff is displayed,
Then the user MUST be prompted to "Deploy or Skip?".

#### Scenario: No Changes
Given `deploy` is run,
And `helm diff` returns no changes (empty output),
Then the deployment MUST be skipped automatically.

#### Scenario: No Interactive
Given `deploy` is run with `--no-interactive`,
Then it MUST proceed with deployment without asking for confirmation.

### Requirement: Helm Diff Configuration
The `helm` configuration section MUST support `diff_enabled` (boolean) and `diff_args` (string).

#### Scenario: Configurable Diff Args
Given `helm.diff_args` is set to "custom args",
When a diff is executed,
Then it should use the provided arguments string with variable interpolation.
