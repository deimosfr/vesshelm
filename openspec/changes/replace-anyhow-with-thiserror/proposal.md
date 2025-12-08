# Replace Anyhow with Typed Errors

## Why
Using `anyhow` makes error handling convenient but opaque. It hides the specific error types, making it difficult for callers to handle specific error cases or for the application to provide precise error reporting. The goal is to move towards typed errors using `thiserror` to improve robustness and maintainability, as requested.

## What Changes
- Enforce the use of `thiserror` for library code and internal modules.
- Refactor existing modules to use `VesshelmError` or specific `thiserror` enums instead of `anyhow::Result`.
- Update `openspec/AGENTS.md` to strictly discourage `anyhow` in future contributions.

## Scope
- `src/clients/*`
- `src/util/*`
- `src/config.rs`
- `src/engine/*`
- `src/lock.rs`
- `AGENTS.md`
