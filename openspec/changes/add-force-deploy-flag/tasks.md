
- [x] Implement exclusion between `--force` and `--dry-run` in `DeployArgs` or `deploy` command
- [x] Verify `vesshelm deploy --force --dry-run` fails

- [x] Add `force` flag to `DeployArgs` struct in `src/cli/commands/mod.rs`
- [x] Update `run` function in `src/cli/commands/deploy.rs` to pass `force` arg
- [x] Update `deploy_chart` signature to accept `force` boolean
- [x] Modify `deploy_chart` logic to bypass "skip if no changes" when `force` is true
- [x] Verify `vesshelm deploy --force` deploys charts even with no changes
- [x] Verify `vesshelm deploy --force` still respects `no_deploy: true`

- [x] Modify `deploy` logic to skip confirmation prompt when `--force` is used
- [x] Verify `vesshelm deploy --force` does not ask for confirmation
