# Warn on Uninstall Dependencies

## Goal
Warn the user when attempting to uninstall a chart that other charts depend on.

## Context
Currently, the `uninstall` command removes a chart without checking if other charts depend on it. This can break deployments that rely on the uninstalled chart.

## Changes
- **Modified Command**: `uninstall`
- **Functionality**:
    - Build a dependency graph of all charts.
    - Check if the target chart has any dependents (reverse dependencies).
    - If dependents exist, display a warning listing them.
    - If no dependents, display a "safe to uninstall" message regarding dependencies.
    - Require explicit confirmation after the warning (existing behavior, just enhanced message).
