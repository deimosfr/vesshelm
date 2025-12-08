# Refactor Config Updater Proposal

## Goal
Move `ConfigUpdater` out of `src/cli/commands/add` to a shared location (`src/util/config_updater.rs`) and incorporate the chart version update logic from `check-updates` (often referred to as sync updates by users) into this shared utility.

## Context
Currently:
- `ConfigUpdater` exists only within the `add` command module.
- `check-updates` command implements its own regex-based logic (`find_and_replace_version`) to update chart versions in `vesshelm.yaml`.

This refactoring will:
- Centralize all `vesshelm.yaml` manipulation logic in one place.
- Reduce code duplication.
- Make the `ConfigUpdater` reusable for other commands.

## Architecture
- **New Location**: `src/util/config_updater.rs`
- **New Capability**: `ConfigUpdater::update_chart_version(content: &mut String, chart_name: &str, new_version: &str) -> Result<()>`
- **Refactoring**:
    - `add` command will import `ConfigUpdater` from `vesshelm::util::config_updater`.
    - `check-updates` command (and potentially future `sync` enhancements) will use `ConfigUpdater` for version updates.
