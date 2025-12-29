# Filter Argument Refactor

## Background
Currently, commands like `sync`, `deploy`, and `check-updates` use an `--only` flag to filter which charts to operate on. This is verbose and less intuitive than standard CLI patterns where positional arguments act as filters.

## Goal
Remove the `--only` flag and allow users to specify charts directly as positional arguments.

## Changes
- Remove `--only` flag from `DeployArgs`, `SyncArgs`, and `CheckUpdatesArgs`.
- Add a positional `charts` argument to these commands.
- Update `src/util/filter.rs` to handle the new argument structure if necessary (logic remains mostly the same, just input source changes).
