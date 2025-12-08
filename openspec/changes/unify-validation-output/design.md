# [Design] Unify Validation Output

## Architecture
The specialized error printing logic in `src/cli/commands/validate.rs` (`print_validation_errors` and `print_field_error`) will be moved to `src/util/error.rs`.
The `format_error` function in `src/util/error.rs` will be enhanced to:
1.  Traverse the `anyhow::Error` chain to find `validator::ValidationErrors`.
2.  Use the rich formatting logic (extracting params like `file`, `name`, `namespace`) to print the error.

## Trade-offs
-   **Pros**: Consistent output, single source of truth for error formatting.
-   **Cons**: `src/util/error.rs` becomes slightly more complex, but purely display logic.

## Dependencies
-   `anyhow` for error chain traversal.
-   `validator` for `ValidationErrors` structure.
