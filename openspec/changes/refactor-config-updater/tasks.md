<!-- id: refactor-config-updater -->
# Refactor Config Updater Tasks

- [x] Move modules <!-- id: 1 -->
    - [x] Create `src/util/config_updater.rs` from existing `add/config_updater.rs`
    - [x] Expose module in `src/util/mod.rs`
    - [x] Update `src/cli/commands/add/mod.rs` to use new path
    - [x] Delete `src/cli/commands/add/config_updater.rs`
- [x] Implement Version Update Logic <!-- id: 2 -->
    - [x] Extract `find_and_replace_version` from `check_updates.rs` to `ConfigUpdater`
    - [x] Rename/Adapt it to `update_chart_version` or similar
    - [x] Move/Adapt unit tests for version matching to `src/util/config_updater.rs`
- [x] Refactor Check Updates <!-- id: 3 -->
    - [x] Update `src/cli/commands/check_updates.rs` to use `ConfigUpdater`
    - [x] Remove local regex logic and tests
- [x] Verification <!-- id: 4 -->
    - [x] Run `cargo test` to ensure all tests pass in new location
    - [x] Manual check: Run `add` command
    - [x] Manual check: Run `check-updates --apply` (mock if needed)
