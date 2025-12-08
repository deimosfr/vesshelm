# Tasks

## Implementation
- [ ] 1. Define `ConfigError` in `src/config/error.rs` and refactor `src/config.rs`
- [ ] 2. Define `ClientError` (and subtypes) in `src/clients/` and refactor clients
- [ ] 3. Define `UtilError` in `src/util/` and refactor util modules
- [ ] 4. Define `AppError` in `src/error.rs` and update `main.rs` / `cli`
- [ ] 5. Remove `anyhow` usage from `src/cli/commands/*.rs`
- [ ] 6. Verify no `anyhow!` macros remain in critical paths
- [ ] 7. Ensure `cargo check` and `cargo test` pass
