# Tasks

- [x] Move validation error printing logic from `src/cli/commands/validate.rs` to `src/util/error.rs`
- [x] Update `format_error` in `src/util/error.rs` to support `anyhow` chain traversal
- [x] Refactor `src/cli/commands/validate.rs` to use `src/util/error.rs`
- [x] Verify `deploy` output matches `validate` output
