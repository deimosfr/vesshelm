# Tasks

- [x] Replace `tracing::info!` with `println!` and `colored` in `src/cli/commands/deploy.rs`.
- [x] Ensure "Skipping" messages are clean and minimal (no timestamps).
- [x] Ensure "Deploying" messages are clean and highlighted.
- [x] Remove `tracing` dependency usage in `deploy.rs` (except maybe for debug/trace levels if hidden by default, but user asked to remove INFO noise).
