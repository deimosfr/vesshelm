# Fix Filtered Progress Count

## Context
When running `vesshelm deploy --only <chart>` or `vesshelm sync --only <chart>`, the CLI filters which charts are processed. However, the progress bar initializes with the *total* number of charts in the configuration, rather than the number of charts that match the filter.

## Problem
This leads to confusing progress feedback. For example, if there are 10 charts and the user deploys or syncs only 1, the progress bar might show `0/10` -> `1/10` and then finish, instead of `0/1` -> `1/1`.

## Solution
Modify the `deploy` and `sync` command logic to:
1.  Apply the `--only` filter to the list of charts *before* initializing the `ProgressTracker` (or calculate the filtered count first).
2.  Initialize the progress bar with the count of charts that will actually be processed (including those that might be skipped/ignored due to internal logic, but excluding those explicitly filtered out by `--only`).
3.  Refactor `sync` loop to match `deploy`'s pattern of pre-filtering for consistency and cleaner code.

## Impact
- **UX**: Progress bar accurately reflects the work to be done.
