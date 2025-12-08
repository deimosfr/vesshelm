# Change: Update Sync Repo Handling

## Why
Users requesting better feedback when syncing charts. Currently, if a repository already exists, the tool silently ignores it or might error depending on implementation details. The user explicitly requested that the `sync` command instruction should not error if there is an existing repo, but just say it already exists.

## What Changes
- Update the `sync` command to check for existing repositories explicitly (or catch the error) and log a friendly message "Repository 'name' already exists" instead of failing or being silent.
- Update the spec to reflect this requirement.

## Impact
- **Specs**: `chart-management` capability modified (Repository Support).
- **Code**: `src/cli/commands/sync.rs`.
