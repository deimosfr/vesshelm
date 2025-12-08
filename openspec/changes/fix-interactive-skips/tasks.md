# Tasks: Fix Interactive Skips

- [x] Add `suspend` method to `ProgressTracker` in `src/util/progress.rs`.
- [x] Refactor `deploy.rs` to use `tracker.suspend` for the confirmation prompt.
- [x] Verify `vesshelm deploy` interactive skips work with progress bar.
