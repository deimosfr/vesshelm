# Design: Co-located Tests

## Background
Currently, the project uses a standard Rust pattern where unit tests are likely in `src/` but integration tests are in `tests/`. The user has explicitly requested that *all* tests be located in the same file as the code.

## Strategy
1.  **Migration**: Existing tests in `tests/` (now temporarily in `src/main.rs`) must be distributed to the specific modules they test.
    - `test_init_*` -> `src/cli/commands/init.rs`
    - `test_sync_*` -> `src/cli/commands/sync.rs`
    - `test_validate_*` -> `src/cli/commands/validate.rs`
2.  **Constraint**: Tests must be located in the *same file* as the logic they verify, typically in a `#[cfg(test)] mod tests` module at the bottom of the file.

## Trade-offs
- **Pros**:
    - High locality: Test code is next to implementation.
    - Access to private functions/structs.
    - clearer ownership of tests per command.
- **Cons**:
    - Files can become larger.

## Implementation Details
- We will identify which command each test in `src/main.rs` (formerly `tests/cli_test.rs`) targets.
- We will move those test functions into the `mod tests` block of the corresponding command file.
- `src/main.rs` should only contain tests for `main.rs` logic (if any), or high-level integration tests that truly don't belong to a sub-command.
