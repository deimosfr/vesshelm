# Refactor Uninstall Logic

This proposal consolidates the `helm uninstall` logic by updating the `uninstall` command to use the shared `HelmClient` implementation.

## Context

The `uninstall` command currently duplicates the `helm uninstall` execution logic manually using `std::process::Command`. The `delete` command now uses a `HelmClient::uninstall` method. This duplication is unnecessary and potentially inconsistent.

## Proposed Changes

1.  **Refactor `uninstall` Command**: Update `src/cli/commands/uninstall.rs` to use `vesshelm::clients::RealHelmClient` (and its `uninstall` method) instead of manually constructing the command.
2.  **Consistency**: Ensure `uninstall` command leverages the "release not found is success" logic from `HelmClient` if appropriate, or handle it consistent with current behavior. (Note: `HelmClient` treats "not found" as success, which is generally desirable for idempotency).
