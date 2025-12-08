## MODIFIED Requirements

### Requirement: OCI Version Check
The `check-updates` command MUST support checking for updates for charts hosted in OCI registries.
It should retrieve the latest available version from the OCI registry and compare it with the configured version.

#### Scenario: Flexible OCI URL Handling
Given a chart configuration with a full OCI URL `oci://ghcr.io/hyperspike/valkey-operator`
And the chart name is `valkey-operator`
When `check-updates` is executed
Then the command MUST query `oci://ghcr.io/hyperspike/valkey-operator`
And MUST NOT duplicate the chart name in the URL (e.g. `.../valkey-operator/valkey-operator`).

#### Scenario: Registry Root OCI URL
Given a chart configuration with repository `oci://ghcr.io/hyperspike`
And chart name `valkey-operator`
When `check-updates` is executed
Then the command MUST query `oci://ghcr.io/hyperspike/valkey-operator`.
