# Optimize Sync Speed with Lockfile

## Context
Currently, `vesshelm sync` downloads and synchronizes all charts every time it runs, even if the charts haven't changed. This can be slow, especially with many charts or slow network connections.

## Problem
Users experience unnecessary delays during `vesshelm sync` because unchanged charts are re-downloaded and re-processed.

## Solution
Introduce a `vesshelm.lock` file that tracks the state of synced charts (version, repository).
On `sync`:
1. Load `vesshelm.lock` if it exists.
2. For each chart in `vesshelm.yaml`:
    - Compare configured version with locked version.
    - Check if the chart destination folder exists locally.
    - If identical AND folder exists, skip sync (unless `--ignore-skip` is used).
    - If different, missing from lockfile, or folder missing, proceed with sync.
3. Update `vesshelm.lock` with the new state after successful sync.

## Impact
- Significantly faster sync for repeated runs.
- Reduced network usage.
- `vesshelm.lock` can be committed to ensure team-wide consistency.
