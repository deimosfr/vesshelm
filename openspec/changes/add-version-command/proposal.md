# Add Version Command and Bump Version

## Context
The project is currently at version `0.1.0`. The user wants to display the app version via a specific command and release version `1.0.0`.

## Problem
- No explicit `version` subcommand exists (only `vesshelm --version` flag).
- Current version is pre-release/alpha `0.1.0`.

## Solution
1. Bump version in `Cargo.toml` to `1.0.0`.
2. Add a `version` subcommand to the CLI that prints the current version (e.g. `vesshelm 1.0.0`).

## Impact
- Clearer versioning for users.
- Official 1.0.0 release milestone.
