# Tasks: Add Take Ownership Flag

- [x] Add `take_ownership` boolean flag to `DeployArgs` in `src/cli/commands/mod.rs`.
- [x] Update `deploy.rs` to append `--take-ownership` to the Helm arguments if the flag is set.
- [x] Verify `helm upgrade` receives the flag (using `--dry-run` or mock).
