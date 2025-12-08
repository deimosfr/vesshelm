# Change: Add Sync Command

## Why
Vesshelm needs the ability to download and synchronize Helm charts locally. This allows users to manage dependencies reliably, supporting offline deployments and ensuring consistent versions across environments.

## What Changes
- Add a new `sync` command to the CLI.
- Introduce configuration support for `repositories`, `charts`, and `destinations` in `vesshelm.yaml`.
- Implement logic to fetch charts from Helm repositories, Git repositories, and OCI registries.
- Implement a "safe replace" strategy: download -> verify/extract -> delete old -> move new.
- Support default and custom destinations for charts.

## Impact
- **Specs**:
    - New `chart-management` capability (sync, fetch logic).
    - New `configuration` capability (schema for repos and charts).
    - Updates to `cli` capability (add `sync` command).
- **Code**: New `cli::commands::sync` module, new `chart` and `repository` modules.
