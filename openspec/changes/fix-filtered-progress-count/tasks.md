# Tasks

- [x] Modify `run` in `src/cli/commands/deploy.rs` to filter `sorted_charts` based on `args.only` *before* creating the `ProgressTracker`.
- [x] Ensure the `deploy` loop iterates over the filtered list.
- [x] Refactor `run` in `src/cli/commands/sync.rs` to filter `config.charts` based on `args.only` *before* creating the `ProgressTracker`.
- [x] Ensure the `sync` loop iterates over the filtered list, mirroring the logic in `deploy.rs`.
