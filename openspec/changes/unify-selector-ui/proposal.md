# Unify Fuzzy Selector UI Consistency

## Context
Currently, the fuzzy selector lists for `deploy`, `delete`, and `uninstall` commands display chart items inconsistently.
- `deploy` and `uninstall` show plain text: `name (namespace)`.
- `delete` attempts to style the namespace: `name (dimmed namespace)`.

This leads to visual inconsistency and potential rendering artifacts (ANSI codes leaking) as seen in user reports.

## Proposed Change
Introduce a standardized method `selection_label()` for the `Chart` struct that returns a consistently formatted and styled string for use in interactive selectors.

### Goals
- **Consistency**: All commands should display charts identically in selection lists.
- **Aesthetics**: Use bold for chart names and dimmed style for namespaces/metadata.
- **Maintainability**: Centralize formatting logic to avoid duplication and diverged styles.

## Design
- Add `pub fn selection_label(&self) -> String` to `Chart`.
- Implementation: `format!("{} ({})", style(&self.name).bold(), style(&self.namespace).dim())`.
- Update `deploy.rs`, `delete.rs`, `uninstall.rs` to use this method.
