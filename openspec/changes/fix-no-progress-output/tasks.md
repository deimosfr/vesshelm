# Tasks: Fix No-Progress Output

- [x] Modify `ProgressTracker` struct in `src/util/progress.rs` to store `no_progress` state.
- [x] Update `ProgressTracker::println` to use `std::println!` when `no_progress` is true.
- [x] Verify `vesshelm deploy --no-progress` still outputs logs/diffs.
