## MODIFIED Requirements

### Requirement: OCI Version Check
The `check-updates` command MUST support checking for updates for charts hosted in OCI registries.
It should retrieve the latest available version from the OCI registry and compare it with the configured version.

#### Scenario: OCI Failure Reporting
Given a configured OCI chart
When `check-updates` attempts to fetch the latest version and fails
Then the command output MUST include the specific error message from the underlying Helm command (stderr).
