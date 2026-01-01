# Fix Delete Summary formatting and logic

This proposal addresses user feedback regarding the `delete` command's summary output.

## Context

The current `delete` command summary:
1.  Combines chart name and namespace on one line.
2.  Displays the parent directory instead of the specific chart directory.
3.  Doesn't explicitly mention the directory deletion in the "Action" line.

## Proposed Changes

1.  **Split Namespace**: Display "Namespace" on its own line in the summary.
2.  **Fix Path**: Append the chart name to the resolved destination to show the full path to be deleted.
3.  **Clarify Action**: explicit text "Delete local directory" in the action summary.
