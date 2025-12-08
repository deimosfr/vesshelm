# Proposal: Add Command

**change-id**: `add-command`

## Summary
Implement a `vesshelm add` command to simplify adding new charts to the configuration. The command will use an interactive wizard to guide the user through adding a chart from an Artifact Hub URL, automatically detecting repository and chart details.

## Motivation
Currently, users must manually edit `vesshelm.yaml` to add charts and repositories. This is error-prone and requires looking up details like repository URLs and chart versions. The `add` command will automate this process, reducing friction and ensuring consistency.

## Solution
The `add` command will:
1.  Prompt the user to select the source type: Artifact Hub (Default), Git, or OCI.
2.  **Artifact Hub**:
    *   Prompt for URL.
    *   Fetch chart details.
3.  **Git**:
    *   Prompt for Git Repository URL.
    *   Prompt for Chart Path within the repo.
    *   Prompt for Version (commit/tag/branch).
4.  **OCI**:
    *   Prompt for OCI URL (e.g., `oci://...`).
    *   Prompt for Version.
5.  Check if the repository exists in `vesshelm.yaml` and add it if missing (detecting type automatically).
6.  Prompt for configuration details (alias, namespace).
7.  Generate the chart configuration block.
8.  Preview the changes and urge the user to confirm.
9.  Update `vesshelm.yaml`.

## Code Quality
To ensure maintainability and robustness:
1.  **Refactoring**: Split `src/cli/commands/add.rs` into smaller, testable modules (e.g., `source` module for different providers, `config_updater` for file manipulations).
2.  **Testing**: Add unit tests for:
    *   URL parsing (Artifact Hub, Git, OCI).
    *   Config update logic (mocking file I/O or using temp files).
