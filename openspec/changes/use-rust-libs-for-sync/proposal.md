# Change: Use Rust Libs for Sync

## Why
The project constraints have been updated to prefer Rust libraries over external binaries (except for `helm`). Current implementation uses `git` CLI and `cp` command, which violates this new constraint and reduces portability.

## What Changes
- Replace `Command::new("git")` with the `git2` crate for cloning and checking out repositories.
- Replace `Command::new("cp")` with native Rust recursive directory copying (using `walkdir` or internal helper).
- Keep `Command::new("helm")` as it is explicitly allowed.

## Impact
- **Dependencies**: Add `git2` and `walkdir` (or `fs_extra`) to `Cargo.toml`.
- **Code**: Refactor `src/cli/commands/sync.rs`.
- **Tests**: Update tests if they relied on mocking `git` binary (integration tests currently mock `helm` but might need update for `git`).
