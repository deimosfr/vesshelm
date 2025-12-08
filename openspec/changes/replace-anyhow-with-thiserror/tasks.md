# Tasks

## Refactoring
- [x] Refactor `src/config.rs` to use `VesshelmError`
- [x] Refactor `src/config/error.rs` (ensure it exists)
- [x] Refactor `src/clients/`
  - [x] `git.rs`
  - [x] `helm.rs`
  - [x] `oci.rs`
  - [x] `artifacthub.rs`
- [x] Refactor `src/util/`
  - [x] `config_updater.rs`
  - [x] `encryption.rs`
  - [x] `interactive.rs` / `interaction.rs`
  - [x] `variables.rs`
  - [x] `dag.rs`
  - [x] `filter.rs`
  - [x] `error.rs` (format_error logic)
- [x] Refactor `src/engine/`
- [x] Refactor `src/cli/commands/` to map typed errors
- [x] Remove `anyhow` from `Cargo.toml` (if possible, or at least restrict usage)
