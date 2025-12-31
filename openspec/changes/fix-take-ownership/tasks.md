# Tasks: Fix Take Ownership Implementation

- [x] Update `deploy_chart` function signature in `src/cli/commands/deploy.rs` to accept `take_ownership: bool`.
- [x] Pass `args.take_ownership` from `run` function to `deploy_chart`.
- [x] Append `--take-ownership` to Helm arguments in `deploy_chart` when the flag is set.
- [x] Add a unit test in `src/cli/commands/deploy.rs` that verifies `interpolate_variables` or `construct_helm_args` (refactored if needed) includes the flag, OR verify via a mock test that the expected command string is generated.
