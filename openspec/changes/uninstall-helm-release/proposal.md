# Uninstall Helm Release on Delete

This proposal adds an optional step to the `delete` command to uninstall the deployed Helm release associated with the chart.

## Context

Currently, `delete` only removes the chart from the local configuration and filesystem. It does not affect the actual release running in the cluster. Users often want to remove the release as well when deleting the chart.

## Proposed Changes

1.  **Interactive Prompt**: Add a prompt asking "Do you also want to uninstall the Helm release?" (default: No).
2.  **Summary Update**: If accepted, show "Uninstall release: Yes" in the summary.
3.  **Execution Logic**:
    *   Attempt `helm uninstall` *first*.
    *   If successful (or release not found), proceed to remove config and files.
    *   If failed, abort the deletion process to preserve config state (safety).
4.  **Client Integration**: Enhance `HelmClient` to support `uninstall`.
