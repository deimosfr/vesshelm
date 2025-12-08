# Sync Command Specification

## MODIFIED Requirements

#### Requirement: Skip Unchanged Charts
The `sync` command MUST skip downloading and processing charts if they are already synced at the correct version, determined by `vesshelm.lock`, AND the destination folder exists locally.

#### Scenario: First run (no lockfile)
Given a valid `vesshelm.yaml` and no `vesshelm.lock`
When `vesshelm sync` is run
Then all charts are synced
And `vesshelm.lock` is created with current versions.

#### Scenario: Second run (no changes)
Given `vesshelm.yaml` and `vesshelm.lock` match versions
And the destination folder exists locally
When `vesshelm sync` is run
Then all charts are skipped
And the output indicates they were skipped due to being up-to-date.

#### Scenario: Local folder missing
Given matching versions in `vesshelm.yaml` and `vesshelm.lock`
And the destination folder is missing locally
When `vesshelm sync` is run
Then the chart is synced (to restore the folder)
And `vesshelm.lock` is updated (or touched).

#### Scenario: Version mismatch
Given `vesshelm.yaml` has version '1.2.0' and `vesshelm.lock` has '1.1.0'
When `vesshelm sync` is run
Then the chart is synced (updated)
And `vesshelm.lock` is updated to '1.2.0'.

#### Scenario: Force sync
Given matching versions in `vesshelm.yaml` and `vesshelm.lock`
When `vesshelm sync --ignore-skip` is run
Then all charts are synced (forcefully)
And `vesshelm.lock` is updated.
