# Update Sync Colors

## Summary
Update the `sync` command output colors to match user preferences for better readability and a consistent "fancy" UI.

## Motivation
The current color scheme for the sync command statuses is not aligned with the desired aesthetic. Specifically, users requested:
- `(up to date)` -> green (currently yellow)
- `(local chart)` -> blue (currently yellow)
- `[SKIP]` status -> grey (currently yellow)

## Proposed Solution
Modify `src/cli/commands/sync.rs` to use:
- `colored::Color::Green` for "up to date" messages.
- `colored::Color::Blue` for "local chart" messages.
- `console::style(...).dim()` or a grey color for `[SKIP]`.
