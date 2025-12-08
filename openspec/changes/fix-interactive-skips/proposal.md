# Fix Interactive Skips

## Context
When using the interactive progress bar, user prompts (via `dialoguer`) can conflict with the progress bar rendering (via `indicatif`) if not handled correctly. The user reports that skips are "not taken into account" when the progress bar is active, likely due to visual corruption or input capturing issues causing unintended confirmations or missing return values.

## Problem
Currently, `deploy_chart` uses `tracker.pause()` which only disables the steady tick. It does not clear the progress bar line. `dialoguer` then prints to stdout, mixing with the progress bar. This can lead to confusing UI where the user thinks they skipped, but maybe the input wasn't registered correctly or logic fell through.

## Solution
Use `indicatif`'s `suspend` mechanism.
- Add `suspend<F, R>(&self, f: F) -> R` to `ProgressTracker`.
- In `deploy.rs`, wrap the confirmation prompt in `tracker.suspend(...)`.
- This ensures the progress bar is explicitly cleared before the prompt and redrawn after, guaranteeing a clean terminal for user interaction.

## Impact
- Reliable interactive prompts.
- Correct skip tracking as user input will be clearly captured.
