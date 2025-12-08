# Implement Chart Deletion

This proposal facilitates the removal of charts from the `vesshelm` configuration and filesystem. It introduces a safer, guided deletion process that handles dependencies and cleans up related resources.

## Context

Currently, `vesshelm` supports syncing, deploying, and "uninstalling" (removing the release from Kubernetes), but it lacks a command to cleanly remove a chart from the local project configuration and storage. Users must manually edit `vesshelm.yaml` and delete directories, which is error-prone.

## Proposed Change

Introduce a `vesshelm delete [CHART_NAME]` command.

### Key Behaviours

1.  **Interactive or Direct**: Users can specify a chart name or select from a list.
2.  **Dependency Safety**: Prevents deletion if other charts depend on the target chart.
3.  **Comprehensive Cleanup**:
    *   Removes the chart directory from the filesystem.
    *   Removes the chart entry from `vesshelm.yaml`.
    *   Removes the chart entry from `vesshelm.lock`.
    *   Removes the associated repository configuration if it becomes unused.
4.  **User Summary**: clearly displays what will be deleted before asking for confirmation.

## Risks

*   **Data Loss**: Deleting files from the filesystem is destructive. The command must be explicit and require confirmation.
*   **Broken Dependencies**: If the dependency check fails or is bypassed, deployments could break. The implementation heavily relies on the existing DAG logic.
