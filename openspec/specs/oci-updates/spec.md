# oci-updates Specification

## Purpose
TBD - created by archiving change oci-check-updates. Update Purpose after archive.
## Requirements
### Requirement: OCI Version Check
The `check-updates` command MUST support checking for updates for charts hosted in OCI registries.
It should retrieve the latest available version from the OCI registry and compare it with the configured version.

#### Scenario: OCI Update Available
Given a configured chart "oci-chart" with version "1.0.0"
And the chart's repository type is "oci"
And the latest version in the OCI registry is "1.1.0"
When I run `vesshelm check-updates`
Then the output should display "Outdated 1.0.0 -> 1.1.0"
And the chart should be marked for update.

#### Scenario: OCI Up to Date
Given a configured chart "oci-chart" with version "1.0.0"
And the chart's repository type is "oci"
And the latest version in the OCI registry is "1.0.0" or lower
When I run `vesshelm check-updates`
Then the output should display "Up to date"
