# Change: Add Only Flag

## Why
Users need to sync or deploy specific charts without running the command for the entire configuration. This is useful for targeted updates, testing single charts, or managing partial deployments.

## What Changes
- Add `--only <chart_name>` argument to `vesshelm sync` and `vesshelm deploy`.
- Support multiple charts via comma-separated values or multiple flags (e.g., `--only chart1,chart2` or `--only chart1 --only chart2`).
- **Sync**: Filter the list of charts to sync. Ensure all specified charts exist; fail otherwise.
- **Deploy**: Filter the deployment list *after* topological sorting. Ensure all specified charts exist; fail otherwise.

## Impact
- **API**: New CLI argument.
- **Code**: `src/cli/commands/mod.rs` (args definitions), `src/cli/commands/sync.rs`, `src/cli/commands/deploy.rs`, `src/util/filter.rs` (new).
- **Behavior**: Partial execution.
