# Check Chart Updates

## Goal
Add a new command to check if new version updates are available for remote charts.

## Context
Currently, users have to manually check for chart updates. This feature adds a command to automate this process, similar to `helm outdated` or `cargo outdated`.

## Changes
- **New Command**: `check-updates` (or `outdated`? User asked for "check if new version update are available". Let's stick to `check-updates` as it implies an action and potentially applying it).
- **Functionality**:
    - Check remote repositories for newer versions of defined charts.
    - Use Semantic Versioning (SemVer) for version comparisons to handle pre-releases and build metadata correctly.
    - Normalize version prefixes (e.g., `v1.2.3` vs `1.2.3`) to avoid false positives.
    - **Non-destructive updates**: specific replacement of version strings in `vesshelm.yaml` without re-serializing the entire file (preserves comments, ordering, and optional fields).
    - Ignore local/git charts with an explicit message.
    - Output format similar to `sync` command.
- **Options**:
    - `--apply`: Update the `vesshelm.yaml` configuration file with the new versions.
    - `--only <chart1,chart2>`: Check or apply updates only for specified charts.

## Out of Scope
- Updating `git` or `local` charts (logic is too complex for now, user explicitly said to ignore them).
