# [Proposal] Unify Validation Output

## Goal Description
Ensure that validation error output is consistent and detailed across all commands, particularly `deploy`. The `deploy` command currently suppresses detailed validation errors (like missing values files) that are visible when running `validate`.

## Proposal
-   Update shared error formatting logic in `src/util/error.rs` to include detailed error extraction logic currently found only in `src/cli/commands/validate.rs`.
-   Update `format_error` to correctly unwrap `anyhow::Error` to find the underlying `ValidationErrors`.
-   Refactor `src/cli/commands/validate.rs` to use the shared logic to avoid duplication.

## Rationale
Users are confused when `deploy` fails with "Values file not found" without specifying which file. `validate` provides this information. Unifying the logic improves UX and maintainability.
