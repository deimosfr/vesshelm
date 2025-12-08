# Proposal: Style Progress Bar Orange

## Context
The user requested aesthetic updates to the progress bar to match a specific visual preference.

## Goal
1.  **Color**: Change the progress bar color to orange (or yellow/gold which provides the orange look in terminals).
2.  **Spacing**: Add a blank line *above* the progress bar to separate it from previous command output.

## Changes

### Styling
- Update `src/util/progress.rs`:
    - Modify `ProgressStyle` to use `.yellow` (closest to orange in standard 8/16 colors) or specific RGB if supported.
    - Print a newline `\n` during initialization or just before the bar is drawn.

### Locations
- `src/util/progress.rs`: `ProgressTracker::new`.

## Risks
- Low. Cosmetic only.
