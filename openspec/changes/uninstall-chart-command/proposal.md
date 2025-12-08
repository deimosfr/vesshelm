# Uninstall Chart Command

## Goal
Add a `vesshelm uninstall <name>` command to uninstall a chart release that is configured in `vesshelm.yaml`, with a confirmation prompt.

## Requirements
1.  **Command Interface**: `vesshelm uninstall <chart_name>`
2.  **Configuration Lookup**: The command must look up `<chart_name>` in `vesshelm.yaml` to resolve the namespace and other potential context.
3.  **Validation**: Error if the chart is not found in `vesshelm.yaml`.
4.  **Confirmation**: Prompt the user for confirmation (e.g., "Are you sure you want to uninstall <name>? [y/N]") before proceeding.
5.  **Action**: Execute `helm uninstall <name> --namespace <namespace>` upon confirmation.

## Non-Goals
-   Modifying `vesshelm.yaml` (removing the entry from config). The request specifies "uninstall one chart", implying the action against the cluster, not the config file itself.

## User Interface
```bash
$ vesshelm uninstall my-chart
? Are you sure you want to uninstall my-chart? [y/N]
```
