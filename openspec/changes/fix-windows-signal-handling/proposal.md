# Fix Windows Signal Handling Compilation

## Goal
Fix the compilation error on Windows caused by the use of Unix-specific signal handling (`tokio::signal::unix`) in the `deploy` command.

## Context
The `vesshelm deploy` command listens for `SIGINT` (Ctrl+C) and `SIGTERM` to gracefully shut down the Helm process. This is currently implemented using `tokio::signal::unix`, which is not available on Windows, causing build failures.

## Solution
Refactor the signal handling logic to be platform-aware.
- **Unix**: Continue using `tokio::signal::unix` to handle both `SIGINT` and `SIGTERM`.
- **Windows**: Use `tokio::signal::ctrl_c` to handle the generic interrupt signal.
- Use `#[cfg(unix)]` and `#[cfg(windows)]` (or `#[cfg(not(unix))]`) to conditionally compile the appropriate logic.

## Risks
- Minor behavior difference: Windows might not catch "termination" signals other than Ctrl+C as precisely as Unix catches SIGTERM, but this is acceptable for basic CLI usage.
