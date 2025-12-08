# Goal
Fix OCI URL construction in `check-updates` command to support flexible repository URLs.

# Context
The user reports `helm show chart failed` when checking updates for an OCI chart where the repository URL already contains the chart name (e.g., `oci://ghcr.io/hyperspike/valkey-operator`). The current implementation naively appends the chart name, resulting in an invalid URL. This mirrors the issue recently fixed in the `sync` command.

# Changes
1.  **Modify `check-updates` command**: Update `src/cli/commands/check_updates.rs` to implement intelligent OCI URL construction (strict path segment matching), consistent with `SyncEngine`.
