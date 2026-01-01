# Refine Delete: Handle Duplicate Chart Names

This proposal refines the `delete` command to handle cases where multiple charts share the same name but exist in different namespaces.

## Context

The initial implementation of `vesshelm delete [NAME]` assumes chart names are unique or picks the first match. In some configurations, users might have multiple instances of the same chart in different namespaces (e.g., `redis` in `dev` and `redis` in `prod`).

## Proposed Change

Modify the chart selection logic in `delete`:

1.  **Ambiguity Detection**: When a user provides a name, check if multiple charts match.
2.  **Disambiguation**: If duplicates exist, prompt the user to select the specific namespace.
3.  **Uniqueness**: Once a name and namespace are selected (or implied), the target is unique.

## Risks

*   **User Confusion**: Adding more prompts might slow down power users, but it prevents accidental deletion of the wrong chart.
