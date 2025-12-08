# Tasks

- [x] Update `HelmConfig` struct in `src/config.rs` with `diff_enabled` and `diff_args`.
- [x] Update `deploy` command arguments in `src/cli/commands/deploy.rs` to accept `--dry-run`.
- [x] Implement `helm diff` execution logic in `deploy.rs`.
- [x] Wire up the conditional logic for dry-run vs deploy-with-diff.
- [x] Update `src/cli/commands/init.rs` to include diff defaults in generated config.
- [x] Verify with unit tests (args construction) and manual test.
- [x] Add `--no-interactive` flag to `deploy` command.
- [x] Implement diff output parsing to detect empty/no-change diffs.
- [x] Implement interactive confirmation prompt ("Deploy or Skip?").
- [x] Update `deploy` logic:
    - If diff is empty -> Skip.
    - If interactive (default) and diff exists -> Prompt.
    - If confirmed or no-interactive -> Deploy.
