# Track Interactive Skips

## Context
Currently, the `vesshelm deploy` command groups all non-deployed charts into a single "Skipped" category. This includes:
1. Charts marked with `no_deploy: true` in configuration.
2. Charts with no changes detected by `helm diff`.
3. Charts explicitly rejected by the user during the interactive confirmation prompt ("Do you want to deploy ...? [y/N]").

## Problem
Mixing these different reasons for skipping makes the summary statistics ambiguous. A user cannot distinguish between deployments that were skipped because they were up-to-date (a success state) versus those they actively chose to ignore (an operational decision).

## Solution
Introduce a new `Ignored` status for interactively skipped deployments.

1. Update `DeployStatus` enum to include `Ignored`.
2. Update `deploy_chart` logic to return `Ignored` when the user answers "No" to the interactive prompt.
3. Update the deployment summary report to show "Ignored" counts separately from "Skipped".
4. "Skipped" will remain for `no_deploy=true` and "No changes detected".

## Impact
- **CLI Output**: The final summary will have an additional line: `Ignored:  <count>`.
- **User Experience**: Clearer feedback on what actually happened during the deployment process.
