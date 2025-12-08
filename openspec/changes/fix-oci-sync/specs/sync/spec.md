## ADDED Requirements

### Requirement: Sync OCI Charts
The system MUST support syncing charts from OCI registries using flexible URL configurations.

#### Scenario: Full OCI URL in Repository Config
Given a configuration with a repository URL `oci://ghcr.io/hyperspike/valkey-operator`
And a chart with name `valkey-operator`
When the sync command is executed
Then the system MUST pull from `oci://ghcr.io/hyperspike/valkey-operator`
And the URL construction MUST detect that `valkey-operator` is the last path segment of the URL.

#### Scenario: Registry Root OCI URL
Given a configuration with a repository URL `oci://ghcr.io/hyperspike`
And a chart with name `valkey-operator`
When the sync command is executed
Then the system MUST pull from `oci://ghcr.io/hyperspike/valkey-operator`
And URL construction MUST append the chart name because `hyperspike` != `valkey-operator`.

#### Scenario: OCI URL Suffix Partial Match
Given a configuration with a repository URL `oci://ghcr.io/my-valkey-operator`
And a chart with name `valkey-operator`
When the sync command is executed
Then the system MUST use `oci://ghcr.io/my-valkey-operator/valkey-operator`
And MUST NOT assume the URL is complete just because it ends with the chart name suffix.
