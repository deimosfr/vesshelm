# Enable Fuzzy Selection for Delete

This proposal adds "type-to-filter" (fuzzy search) capabilities to the chart selection prompt in the `delete` command.

## Context

Currently, `delete` uses a standard list selection. Users with many charts find it difficult to locate the specific chart they want to remove without filtering.

## Proposed Changes

1.  **Switch to FuzzySelect**: Replace `dialoguer::Select` with `dialoguer::FuzzySelect` in `src/cli/commands/delete.rs`.
2.  **Enable Feature**: Ensure `dialoguer` reference in `Cargo.toml` has `fuzzy-select` feature enabled.
3.  **UX Improvement**: Allow users to type to filter the list of charts.
