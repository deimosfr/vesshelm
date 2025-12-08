## Context
The project uses `anyhow` which provides easy error propagation but poor type safety for consumers (even internal ones).
We need to transition to distinct error types per domain (configuration, clients, utilization, CLI).

## Goals
- Type-safe errors using `thiserror`.
- Meaningful error variants for specific failure modes (e.g., `ConfigError::NotFound`, `HelmError::CommandFailed`).
- Eliminate `anyhow::Result` from internal APIs.

## Non-Goals
- We might still use `anyhow` in `main.rs` strictly for top-level printing if `AppError` doesn't implement `Termination` or similar, but the goal is to "not see anyhow anymore" in the core logic.

## Decisions

### Module-Specific Errors
We will define error enums closer to where they are generated:
- `src/config/error.rs` -> `ConfigError`
- `src/clients/error.rs` -> `ClientError` (or `HelmError`, `GitError`, `OciError`)
- `src/cli/error.rs` -> `CliError`

### Global AppError
A top-level `AppError` enum in `src/error.rs` will aggregate these for the CLI's main entry point, likely implementing `From` conversions.
```rust
#[derive(Debug, Error)]
pub enum AppError {
    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),
    #[error("Client error: {0}")]
    Client(#[from] ClientError),
    // ...
}
```

### Validation Integration
We will integrate `validator::ValidationErrors` into `ConfigError`.
