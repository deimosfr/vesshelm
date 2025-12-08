# Fix No-Progress Output

## Context
When running `vesshelm deploy --no-progress`, the expectation is that the progress bar (spinner, animation) is hidden, but essential logs (diffs, status messages) are still visible.

## Problem
Currently, `ProgressTracker` uses `ProgressBar::hidden()` when `no_progress` is set. It seems that `self.pb.println()` on a hidden bar may suppress output or not behave as a standard stdout pass-through, causing diffs and other logs to be lost.

## Solution
Update `ProgressTracker` to be aware of the `no_progress` state.
- When `no_progress` is true, `println` should write directly to standard output (using `println!`) instead of delegating to the hidden progress bar's `println` method.
- Ensuring that meaningful output is preserved even in "quiet" mode (which is just "no animation" mode here).

## Impact
- Correct logging behavior in CI/CD or non-interactive environments where `no_progress` is often used.
