- [x] **Core: Configuration**
  - [x] Rename `HelmConfig` struct to `VesshelmConfig` in `src/config.rs`.
  - [x] Rename fields in `Config` struct: `helm` -> `vesshelm`.
  - [x] Rename fields in `VesshelmConfig` struct: `args` -> `helm_args`.
  - [x] Update Serde attributes to match new YAML keys.

- [x] **Integration: Codebase Updates**
  - [x] Update `src/cli/commands/deploy.rs` to use new config path.
  - [x] Update `src/engine/sync.rs` (if applicable).
  - [x] Fix any other compilation errors resulting from the rename.

- [x] **Documentation**
  - [x] Update `README.md` configuration reference.
  - [x] Update `README.md` configuration examples.

- [x] **Tests**
  - [x] Update unit tests in `src/config.rs`.
  - [x] Update integration tests / mocks that verify config loading.
