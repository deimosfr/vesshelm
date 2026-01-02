# Refactor Configuration Structure

## Why
The current configuration structure places global execution arguments under a generic `helm` block. As Vesshelm grows, we want a more specific namespace for tool-wide configuration. Moving `helm` to `vesshelm` and renaming `args` to `helm_args` clarifies that these are Vesshelm's wrapper arguments around Helm, not just raw Helm config.

## Solution Overview
1.  Rename the top-level configuration key `helm` to `vesshelm`.
2.  Rename the sub-key `args` to `helm_args`.
3.  Retain `diff_enabled` and `diff_args` within the new `vesshelm` block.
4.  Update the `Config` struct and all references in the codebase.
5.  Update documentation and examples to reflect the new structure.

## Scope
-   **Configuration**: `vesshelm.yaml` structure change.
-   **Code**: `src/config.rs` and usages in `deploy.rs`, `sync.rs`, etc.
-   **Docs**: README and example configs.
