# Tasks

- [x] Define spec delta for `uninstall` command <!-- id: 0 -->
    - [x] Create `openspec/changes/uninstall-chart-command/specs/cli-commands/spec.md` <!-- id: 1 -->
- [x] Implement `uninstall` command <!-- id: 2 -->
    - [x] Create `src/cli/commands/uninstall.rs` with `run` function <!-- id: 3 -->
    - [x] Update `src/cli/commands/mod.rs` to export `uninstall` module <!-- id: 4 -->
    - [x] Update `src/main.rs` to add `Uninstall` enum variant to `Commands` and handle dispatch <!-- id: 5 -->
    - [x] Implement chart lookup logic in `uninstall.rs` using `crate::config` <!-- id: 9 -->
    - [x] Implement confirmation prompt using `inquire` or `dialoguer` (check `src/cli/util` if exists or other commands) <!-- id: 10 -->
    - [x] Implement `helm uninstall` execution using `std::process::Command` <!-- id: 11 -->
- [/] Verify implementation <!-- id: 6 -->
    - [ ] Run `cargo build` to ensure no compilation errors <!-- id: 7 -->
    - [ ] Run `vesshelm uninstall --help` to check argument parsing <!-- id: 8 -->
    - [ ] Manual test: Create a dummy `vesshelm.yaml`, run `vesshelm uninstall <dummy>`, verify prompt and helm command (dry-run/mock if possible) <!-- id: 12 -->
