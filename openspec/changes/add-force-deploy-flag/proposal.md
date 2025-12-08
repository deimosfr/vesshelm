# Add Force Flag to Deploy Command

## Summary
Add a `--force` flag to the `deploy` command to bypass the "skip if no changes" check deployment confirmation. This ensures that charts are deployed even if the diff shows no changes, and does not ask for confirmation, unless they are explicitly marked with `no_deploy: true` in the configuration. Additionally, enforce that `--force` cannot be used with `--dry-run`.

## Problem Code
Currently, `deploy` skips charts if `helm diff` returns no output (indicating no changes). Users may want to force a redeployment (e.g., to trigger hooks, restart pods, or ensure consistency) regardless of the diff status.

## Solution
1.  Add `--force` (alias `-f`?) to `DeployArgs` struct.
2.  Pass this flag into `deploy_chart` function.
3.  In `deploy_chart`, modify the diff check logic: if `force` is true, proceed to deploy even if diff is empty.
4.  Ensure `no_deploy: true` in `vesshelm.yaml` still takes precedence (skips deployment regardless of `--force`).
5.  Prevent `--force` and `--dry-run` from being used together (mutex).

## Impact
- **Deploy Command**: New flag available. Behavior changes only when flag is used.
- **Charts**: No config changes required for charts.
