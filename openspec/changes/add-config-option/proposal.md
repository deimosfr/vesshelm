# Add Global Config Option

## Goal
Add a global `--config` option to specify the path to the configuration file, defaulting to `vesshelm.yaml`.

## Requirements
1.  **Global Option**: The CLI MUST accept a `--config <path>` (or `-c <path>`) argument at the top level.
2.  **Default Value**: If not specified, the default value MUST be `vesshelm.yaml` (in current directory).
3.  **Propagation**: All commands that read or write the configuration file (`deploy`, `sync`, `validate`, `graph`, `uninstall`, `init`) MUST use the specified path.
4.  **Init Command**: The `init` command should check/create the file at the specified path.

## Non-Goals
-   Supporting multiple config files at once (merging).
-   Environment variable overrides (unless requested later, sticking to CLI flag for now).
