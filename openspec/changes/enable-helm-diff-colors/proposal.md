# Enable Helm Diff Colors

## Context
The `helm diff` plugin output is currently monochrome when run via `vesshelm deploy`, making it harder to read changes.

## Problem
The `vesshelm` CLI captures stdout/stderr, which often causes downstream tools like `helm diff` to disable color output automatically as they detect a non-interactive terminal.

## Solution
Force color output for `helm diff` by:
1. Setting the `HELM_DIFF_COLOR` environment variable to `true` when executing the command.
2. Explicitly passing `--color` flag to `helm diff` if supported/needed (Environment variable is usually sufficient and safer for older versions).

## Impact
- Improved readability of deployment diffs.
