## ADDED Requirements

#### Scenario: Skipping Deployment
- **Given** a chart configured with `no_deploy: true` in `vesshelm.yaml`
- **When** I run `vesshelm deploy`
- **Then** the chart should NOT be deployed to the cluster.
- **And** a message indicating it was skipped due to `no_deploy` should be displayed.

#### Scenario: Syncing No-Deploy Charts
- **Given** a chart configured with `no_deploy: true` in `vesshelm.yaml`
- **And** `no_sync` is false (or omitted)
- **When** I run `vesshelm sync`
- **Then** the chart SHOULD be synced (downloaded/updated) normally.
