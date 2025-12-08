# Tasks

- [x] Define spec delta for configuration option <!-- id: 0 -->
    - [x] Create `openspec/changes/add-config-option/specs/cli/spec.md` <!-- id: 1 -->
- [x] Implement `--config` option <!-- id: 2 -->
    - [x] Update `Cli` struct in `src/main.rs` to add `#[arg(long, global = true, default_value = "vesshelm.yaml")] config: String` <!-- id: 3 -->
    - [x] Create config loader helper `crate::config::load_from_path(path: &Path) -> Result<Config>` <!-- id: 4 -->
    - [x] Refactor `deploy` to accept `config_path: &Path` instead of reading "vesshelm.yaml" directly <!-- id: 5 -->
    - [x] Refactor `sync` to accept `config_path: &Path` <!-- id: 6 -->
    - [x] Refactor `validate` to accept `config_path: &Path` <!-- id: 7 -->
    - [x] Refactor `graph` to accept `config_path: &Path` <!-- id: 8 -->
    - [x] Refactor `uninstall` to accept `config_path: &Path` <!-- id: 9 -->
    - [x] Refactor `init` to accept `config_path: &Path` for writing <!-- id: 10 -->
    - [x] Update `main.rs` to pass `cli.config` path to all commands <!-- id: 11 -->
- [x] Verify implementation <!-- id: 12 -->
    - [x] Build project <!-- id: 13 -->
    - [x] Test `vesshelm --config my-vesshelm.yaml validate` (with renamed file) <!-- id: 14 -->
    - [x] Test default `vesshelm validate` works as before <!-- id: 15 -->
