# Proposal: Update Progress Bar UX

## Context
The persistent progress bar provides great visibility, but introduces two issues:
1.  **Interference with Prompts**: The progress bar hides or overwrites the interactive "Do you want to deploy?" dialog, making it impossible for the user to respond.
2.  **Visual Separation**: The final summary output blends too closely with the last log line.
3.  **Disable Option**: Users may want to disable the bar entirely using `--no-progress`.

## Goal
1.  **Interactive Compatibility**: Ensure the progress bar is hidden/cleared before the interactive prompt to avoid visual confusion.
2.  **Disable Progress**: Add `--no-progress`.
3.  **Visual Separation**: Add blank line before summary.

## Changes

### Interactive Prompts
- Update `src/util/progress.rs`: Adjust `tracker` logic to allow printing above the bar or pausing updates without clearing.
- Update `src/cli/commands/deploy.rs`: Insert blank line and ensure progress bar is cleared before prompt is shown.

### CLI & UI (Previous Scope)
- `no_progress` flag (already implemented).
- Blank line (already implemented).

## Risks
- Low. Fixes a critical UX bug.
