# Centralize Config Update Spec

## Goal
Centralize `vesshelm.yaml` modification logic into a single `ConfigUpdater` utility.

## ADDED Requirements

### Requirement: Shared ConfigUpdater
The application MUST provide a shared `ConfigUpdater` utility in `src/util/config_updater.rs` that encapsulates all `vesshelm.yaml` modification logic.

#### Scenario: Add Command Uses Shared Utility
Given the `add` command is executed
When the user confirms adding a new chart
Then the `ConfigUpdater` from `src/util` is used to append the new entry.

#### Scenario: Check Updates Command Uses Shared Utility
Given the `check-updates` command is executed with `--apply`
When an update is found
Then the `ConfigUpdater` from `src/util` is used to update the version in place.

### Requirement: Version Update Logic
The `ConfigUpdater` MUST provide a method `update_chart_version` that updates a specific chart's version in the configuration file without disrupting comments or other fields.

#### Scenario: Update Version Preserves Comments
Given a `vesshelm.yaml` with comments on a chart version line
When `update_chart_version` is called
Then the version number is updated
And the comments on that line remain intact.

#### Scenario: Update Version Respects Scope
Given a `vesshelm.yaml` with multiple charts having the same version key
When `update_chart_version` is called for "chart-A"
Then only "chart-A"'s version is updated
And "chart-B" remains unchanged.
