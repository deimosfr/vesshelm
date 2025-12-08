# Change: Add Check Update Summary

## Why
Users want to quickly see how many charts are outdated, skipped, or up-to-date after running `check-update`.
This brings the command inline with other commands like `sync` that provide a summary.

## What Changes
- Update `check-update` command to track statistics (checked/outdated, skipped, errors).
- Print a summary table/block at the end of execution.

## Impact
- Affected specs: `check-update`
- Affected code: `src/cli/commands/check_updates.rs`
