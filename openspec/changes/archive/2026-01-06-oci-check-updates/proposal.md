# OCI Check Updates

## Why
Currently, `vesshelm check-updates` skips OCI charts with a message "Skipped (local/git/oci)". OCI is becoming a standard for Helm chart distribution. Users managing OCI charts need to know when updates are available just like for standard repositories.

## What Changes
- Update `check_updates.rs` to handle `RepoType::Oci`.
- Implement a strategy to fetch the latest version for OCI charts.
  - Since `helm search repo` doesn't work for OCI, we will use `helm show chart <oci-url>` which retrieves the metadata of the *latest* available version by default (if no version is pinned in the command).
  - Comparing this returned version against the configured version allows detecting updates.
