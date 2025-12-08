# Check Updates Capability

## ADDED Requirements

### Requirement: Support Pre-Release Versions

The system MUST support checking for updates even if the chart only has pre-release versions available (versions with a hyphen, e.g., `3.9.2-1`).

#### Scenario: Pre-Release Only Chart

Given a chart `mealie` that only has pre-release versions in the repository
When I run `vesshelm check-updates`
Then it should find the pre-release version
And it should not fail with "Chart not found"
