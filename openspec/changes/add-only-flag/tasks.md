## 1. Implementation
- [x] 1.1 Modify `src/cli/commands/mod.rs`:
    - Define `SyncArgs` struct with `only` field.
    - Update `DeployArgs` struct with `only` field.
    - Update `Commands::Sync` variant to hold `SyncArgs`.
- [x] 1.2 Modify `src/main.rs`:
    - Update `Commands::Sync` match arm to pass args to `sync::run`.
- [x] 1.3 Modify `src/cli/commands/sync.rs`:
    - Update signature to accept `SyncArgs`.
    - Implement filtering loop.
- [x] 1.4 Modify `src/cli/commands/deploy.rs`:
    - Implement post-sort filtering.
- [x] 1.5 Verification:
    - `cargo run -- sync --only <chart>`
    - `cargo run -- deploy --only <chart> --dry-run`

## 2. Validation Enhancement
- [x] 2.1 Refactor filtering logic to validate existence:
    - In `sync.rs` and `deploy.rs` (or shared utility), check if all items in `only` list exist in the full chart list.
    - Collect missing chart names.
    - Return specific error if any are missing.
- [x] 2.2 Verify missing chart error:
    - `cargo run -- sync --only non-existent` -> Error
    - `cargo run -- deploy --only valid,non-existent` -> Error

## 3. Refactoring & Testing
- [x] 3.1 Create `src/util/filter.rs`:
    - Implement `validate_only_args(all_charts, only_args) -> Result<()>`.
    - Add unit tests covering: empty only, valid only, partial valid, invalid only (case sensitivity?).
- [x] 3.2 Integrate Shared Function:
    - Use `vesshelm::util::filter::validate_only_args` in `sync.rs`.
    - Use `vesshelm::util::filter::validate_only_args` in `deploy.rs`.
