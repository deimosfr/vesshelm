# Capability: Co-located Testing

## ADDED Requirements

### Requirement: Tests must be co-located with implementation
All tests (unit and integration-style) MUST be defined within the same source file as the logic they are testing. `tests/` directory MUST NOT be used. Tests for a specific command MUST go into that command's source file.

#### Scenario: Adding a new function
- Given I add a new function `fn calculate()` in `src/math.rs`
- When I write tests for it
- Then I must add a `#[cfg(test)] mod tests` block inside `src/math.rs`
- And I must not create `tests/math_test.rs`

#### Scenario: Migrating existing tests
- Given I have `src/main.rs` containing tests for `init`, `sync`, and `validate`
- When I apply this change
- Then `test_init_*` must be moved to `src/cli/commands/init.rs`
- And `test_sync_*` must be moved to `src/cli/commands/sync.rs`
- And `test_validate_*` must be moved to `src/cli/commands/validate.rs`
