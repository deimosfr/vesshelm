# Change: Factorize Command Summary

## Why
Currently, multiple commands (`sync`, `deploy`, `check-update`, `add`, `delete`) implement their own summary output logic. This leads to inconsistency in styling (bolding, coloring, newlines) and logic (alignment, structure).
Refactoring this into a shared utility will ensure a consistent user experience and reduce code duplication.

## What Changes
- Create a shared summary utility (e.g., `src/util/ui/summary.rs`).
- Update all distinct commands to utilize this shared utility for printing their final summary.
- Enforce consistent styling (header bold/style, key alignment, value coloring).

## Impact
- Affected specs: `check-update`, `sync`, `deploy`, `add`, `delete`
- Affected code: `src/cli/commands/*.rs`, `src/util/ui.rs`
