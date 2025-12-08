## ADDED Requirements

#### Scenario: Forcing Deployment
- **Given** a chart that has no changes compared to the cluster state
- **When** I run `vesshelm deploy --force`
- **Then** the chart SHOULD be deployed (upgraded) regardless of the diff result.
- **And** the user SHOULD NOT be asked for confirmation.

#### Scenario: Force Skips Confirmation
- **Given** a chart update is pending
- **When** I run `vesshelm deploy --force`
- **Then** the deployment SHOULD proceed without asking "Do you want to deploy...?"

#### Scenario: Force Respects No-Deploy
- **Given** a chart configured with `no_deploy: true` in `vesshelm.yaml`
- **When** I run `vesshelm deploy --force`
- **Then** the chart should NOT be deployed.
- **And** it should be skipped with the usual `no_deploy` message.

#### Scenario: Force Conflicts with Dry Run
- **When** I run `vesshelm deploy --force --dry-run`
- **Then** the command SHOULD fail with an error indicating that `--force` and `--dry-run` cannot be used together.
