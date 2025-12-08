# Change: Add Init Command

## Why
Vesshelm relies on Helm for chart management. Users need a simple way to verify that their environment is correctly set up with all dependencies and to bootstrap a default configuration.

## What Changes
- Add a new `init` command to the CLI.
- The command will check for the presence of the `helm` executable.
- The command will create a default configuration file if one does not exist.

## Impact
- **Specs**: New `initialization` capability.
- **Code**: New `cli::commands::init` usage.
