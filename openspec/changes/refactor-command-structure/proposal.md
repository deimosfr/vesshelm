# Change: Refactor Command Structure

## Why
The project constraints have been updated (`openspec/project.md`) to require clear separation of commands. Currently, all command logic resides in `src/cli/commands.rs`, which violates this constraint.

## What Changes
- Refactor `src/cli/commands.rs` into a directory `src/cli/commands/`.
- Create `src/cli/commands/mod.rs` to expose the modules.
- Create `src/cli/commands/init.rs` for the initialization logic.
- Create `src/cli/commands/sync.rs` for the synchronization logic.
- Update `src/main.rs` and `src/cli/mod.rs` to reflect the new structure.
- **Fix**: Improve `sync` command robustness (handle existing repos gracefully) to address reported runtime errors.

## Impact
- **Code**: `src/cli/commands.rs` (deleted), `src/cli/commands/*` (new).
- **Functionality**: No change in behavior, purely structural (plus robustness fix).
