## 1. Implementation
- [x] 1.1 Update `SyncArgs` in `src/cli/commands/mod.rs` to include `jobs`.
- [x] 1.2 Update `SyncOptions` in `src/engine/sync.rs` to include `jobs`.
- [x] 1.3 Refactor `SyncEngine::sync` to use `tokio::task::JoinSet` or similar for concurrency.
- [x] 1.4 Ensure `helm_client` and `git_client` are safely shared.
- [x] 1.5 Update `observer` pattern to use a channel for thread-safe UI updates.
- [x] 1.6 Verify `lockfile` updates are thread-safe.
- [x] 1.7 Add tests for parallel execution.

## 2. Check Updates Implementation
- [x] 2.1 Update `CheckUpdatesArgs` in `src/cli/commands/mod.rs` to include `jobs`.
- [x] 2.2 Refactor `check_updates` command logic to process charts concurrently.
- [x] 2.3 Ensure concurrent output is handled correctly (e.g. using channels/UI observers).
- [x] 2.4 Verify correctness of OCI/Helm client concurrent usage.
