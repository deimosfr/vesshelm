# Change: Add Local Charts Support

## Why
Users want to deploy charts that are available locally on the filesystem without needing to define a repository or version. This simplifies development and usage of private/ad-hoc charts.

## What Changes
- Modify `Chart` configuration to make `repo_name` and `version` optional.
- Update `validate` command to skip repository existence checks for local charts.
- Update `sync` command to skip local charts.
- Update `deploy` command to resolve local chart paths correctly (likely using `chart_path` or defaulting to destination).

## Impact
- **Configuration**: Schema change (fields become optional). Backward compatible for valid configs.
- **Workflow**: `sync` skips local charts. `deploy` works with local paths.
