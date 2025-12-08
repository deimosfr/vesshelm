# Change: Parallelise Sync Process

## Why
Currently, the `sync` and `check-update` processes check and download charts sequentially. This can be slow when there are many charts.
Parallelising these processes will significantly speed up the `sync` and `check-update` commands.

## What Changes
- Add `--jobs` / `-j` argument to `sync` and `check-update` commands (default 5).
- Update `SyncEngine` to process charts concurrently using `tokio`.
- Ensure output remains readable despite concurrency.
- Refactor `SyncEngine` to use channels for event reporting.

## Impact
- Affected specs: `sync`, `check-update`
- Affected code: `src/cli/commands/mod.rs`, `src/cli/commands/sync.rs`, `src/cli/commands/check_updates.rs`, `src/engine/sync.rs`
