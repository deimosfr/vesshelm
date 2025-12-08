## MODIFIED Requirements

### Requirement: OCI Version Check
The `check-updates` command MUST support checking for updates for charts hosted in OCI registries.
It should retrieve the latest available version from the OCI registry and compare it with the configured version.

#### Scenario: Missing Latest Tag
Given an OCI chart repository that does not publish a `latest` tag
But has semantic version tags (e.g. `v0.0.1`, `v0.0.2`)
When `check-updates` is executed
Then the command MUST list all available tags from the registry
And MUST identify the highest semantic version as the latest version.
