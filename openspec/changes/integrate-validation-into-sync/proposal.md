# Change: Integrate Validation into Sync

## Why
To ensure data integrity and prevent runtime errors during the synchronization synchronization process, the `sync` command should validate the configuration before attempting any operations. By sharing a "validated configuration" structure, the `sync` logic can be simplified, avoiding redundant checks that are already covered by the validation phase.

## What Changes
- Refactor configuration loading to separate "parsing" from "validation".
- Introduce a mechanism (e.g., `ValidatedConfig` struct or just convention) where `sync` operations accept a config that is guaranteed to be valid.
- Update `sync` command to:
    1. Parse config.
    2. Run validation (reusing logic from `add-validate-command`).
    3. If valid, proceed with sync using the validated data.
    4. If invalid, exit with error details.
- Remove ad-hoc checks in `sync` that are covered by validation (e.g., "repo not found" checks inside the loop can be assumed safe if validation ensures referential integrity).

## Impact
- **Dependencies**: Depends on `add-validate-command` logic (shared validation code).
- **Code**: `src/cli/commands/sync.rs`, `src/config.rs`.
