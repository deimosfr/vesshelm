# Improve Error Context

## Goal
Improve error messages throughout the configuration loading and updating process by adding explicit context (file paths, chart names) to error variants that currently lack them.

## Context
Several `ConfigError` variants provide generic error messages without sufficient context to help the user locate the source of the problem. For example:
- `ParseFailed` does not state which file failed to parse.
- `DefaultDestNotFound` does not indicate which chart triggered the error.
- `VersionBoundaryCrossed` does not specify which chart was being updated.
- `FileWriteFailed` does not include the file path.
- `ValidationFailed` is generic for other validation errors.

## Solution
Update `ConfigError` variants to include necessary context strings (paths, names) and update call sites to provide this information.
