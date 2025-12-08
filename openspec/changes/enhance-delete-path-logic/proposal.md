# Enhance Delete Path Resolution

This proposal improves how the `delete` command determines the target directory for a chart, particularly when custom destinations or local chart paths are used.

## Context

Currently, `delete` assumes the target path is always `resolved_destination + / + chart_name`. This fails for:
1.  **Local Charts**: Where `chart.chart_path` is the source of truth.
2.  **Explicit Destinations**: Where `chart.dest` might point directly to the chart folder, not a parent.

## Proposed Changes

1.  **Local Chart Support**: If `repo_name` is missing and `chart_path` is set, use `chart_path` as the delete target.
2.  **Explicit Destination Adaptation**: If `dest` is explicitly defined (and not a known alias), consider it as the potential full path.
    *   Validation logic: Check if `dest` exists vs `dest/name`.
    *   Prioritize consistency but allow "adaptation" if `dest` seems to be the target.
3.  **Directory Status**: Explicitly check if the calculated path exists.
    *   Summary should say "Status: Present (will be deleted)" or "Status: Missing (already gone)".
4.  **Test Coverage**: Add unit tests for these path resolution scenarios.
