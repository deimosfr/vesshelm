# Tasks: Optimize Sync with Lockfile

- [x] Define `Lockfile` and `SyncedChart` structs in a new `src/lock.rs` module.
- [x] Implement `Lockfile::load()` and `Lockfile::save()` methods.
- [x] Add `--ignore-skip` flag to `SyncArgs` in `src/cli/commands/mod.rs`.
- [x] Update `sync` command logic in `src/cli/commands/sync.rs` to use `Lockfile` for skipping.
- [x] Update `sync` command to update and save `Lockfile` upon completion.
- [x] Add integration tests in `tests/` to verify lockfile creation and skip behavior.
- [x] Verify local destination folder existence before skipping sync (even if locked).
