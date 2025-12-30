## ADDED Requirements

### Requirement: Support apply-sync flag
The `check-updates` command SHALL support an `--apply-sync` flag which, when provided, will apply any found updates to `vesshelm.yaml` and subsequently trigger the `sync` process to reconcile the cluster state.

#### Scenario: User applies updates and syncs in one go
Given a `vesshelm.yaml` with outdated charts
And I run `vesshelm check-updates --apply-sync`
Then the `vesshelm.yaml` file is updated with the latest versions
And the `sync` command is executed to deploy the changes
