---
status: draft
change-id: refactor-error-handling
---

# Change: Refactor Error Handling

## Why
The codebase currently relies heavily on `anyhow::Result`, which obscures error types and makes handling specific failure cases difficult.
The user explicitly requested "clean, clear error enums" and the removal of `anyhow` from the return signatures.

## What Changes
- Introduce typed error enums using `thiserror` for all modules (`ConfigError`, `ClientError`, `CliError`, etc.).
- Refactor function signatures to return `Result<T, ModularError>` instead of `Result<T, anyhow::Error>`.
- Remove `anyhow!` macro usage in favor of specific error variants.
- Implement a global `AppError` enum if necessary to aggregate errors at the top level for the CLI.

## Impact
- Affected specs: `specs/error-handling/spec.md`
- Affected code: Entire codebase (`src/**/*.rs`)
- **BREAKING**: Function signatures will change throughout the project.
