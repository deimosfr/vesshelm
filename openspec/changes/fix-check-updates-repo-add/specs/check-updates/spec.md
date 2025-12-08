# Check Updates Capability

## ADDED Requirements

### Requirement: Ensure Repositories are Registered

The system MUST ensure that all repositories defined in the configuration are registered with the underlying Helm client before attempting to check for updates.

#### Scenario: New Repository

Given a `vesshelm.yaml` with a new repository `deimosfr-charts` that is not yet added to Helm
And a chart `mealie` from this repository
When I run `vesshelm check-updates`
Then it should automatically add the `deimosfr-charts` repository
And it should successfully find and check the `mealie` chart version
And it should not fail with "Chart not found"
