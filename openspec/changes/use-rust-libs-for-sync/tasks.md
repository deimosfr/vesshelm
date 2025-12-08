## 1. Implementation
- [x] 1.1 Add `git2` (and potentially `fs_extra` or `walkdir`) to `Cargo.toml`
- [x] 1.2 Refactor `src/cli/commands/sync.rs` to use `git2` for Git repo fetching
- [x] 1.3 Refactor `src/cli/commands/sync.rs` to use native Rust for file copying/moving
- [x] 1.4 Update integration tests (`tests/cli_test.rs`)? (Mocking `git` binary won't work anymore if we use `git2`, might need to mock a real git repo or use a test fixture)
- [x] 1.5 Verify `sync` command works with real/test repos
