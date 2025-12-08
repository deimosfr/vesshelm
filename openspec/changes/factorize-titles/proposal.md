---
status: draft
change-id: factorize-titles
---

# Change: Factorize Console Titles

## Why
Currently, deploy/delete/uninstall commands use inconsistent and hardcoded patterns like `.println(&format!(style("==>")...))`.
This makes the UI disjointed and harder to maintain or change globally.

## What Changes
- Factorize title formatting into a dedicated `ui::style_step` function.
- Replace manual formatting in `src/cli/commands/{deploy,delete,uninstall}.rs` with the new function.

## Impact
- Affected specs: `specs/console-titles/spec.md`
- Affected code: `src/util/ui.rs`, `src/cli/commands/`
