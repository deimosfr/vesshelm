# Chart Management

## MODIFIED Requirements

### Sync Charts
The `sync` command downloads charts to the local filesystem.

#### Scenario: Syncs charts from updated repositories
Given a `vesshelm.yaml` with remote charts
And the local helm repository cache is outdated
When I run `vesshelm sync`
Then it should run `helm repo update` before pulling
And it should successfully pull the charts
