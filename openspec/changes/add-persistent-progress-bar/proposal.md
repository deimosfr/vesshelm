# Proposal: Persistent Progress Bar for Deploy and Sync

## Context
Currently, the `sync` command uses a spinner for each individual chart, which is cleared after completion. The `deploy` command uses simple log statements. Users have requested a "fancy", persistent bottom bar showing overall percentage advancement to improve the user experience and give better feedback on long-running operations.

## Goal
Implement a persistent progress bar at the bottom of the terminal output for both `sync` and `deploy` commands.
- The bar should show the overall progress (e.g., "Processing [2/5] ...").
- It should remain visible at the bottom while logs scroll above it.
- It should look "fancy" and "super clean" (aesthetic improvements).
- It should avoid glitches.

## Changes
We will modify the CLI output handling to utilize `indicatif`'s capabilities for persistent scroll-friendly progress bars.

### Risk
- Terminal compatibility issues (though `indicatif` handles this well).
- Conflict with existing logging if not handled via the progress bar's print facility.
