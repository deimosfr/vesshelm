# Add --take-ownership Flag

## Context
Users sometimes encounter ownership conflicts when deploying charts (e.g. "invalid ownership metadata" or resource already exists). Some Helm setups or plugins might support a `--take-ownership` flag to resolve this.

## Problem
Currently, there is no way to pass this specific flag to the underlying `helm` command via `vesshelm deploy` without modifying the global config or `vesshelm.yaml`.

## Solution
Add a `--take-ownership` boolean flag to `vesshelm deploy`.
When provided, it appends `--take-ownership` to the Helm command arguments generated for each chart.

## Impact
- Allows users to resolve ownership issues on the fly properly.
- No config changes required for one-off fixes.
