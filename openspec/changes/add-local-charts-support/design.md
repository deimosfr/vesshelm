# Design: Local Charts Support

## Configuration Changes
- `repo_name`: Change from `String` to `Option<String>`.
- `version`: Change from `String` to `Option<String>`.
- **Validation**:
  - If `repo_name` is `None`, skip repo existence check.
  - If `repo_name` is `None`, we could enforce `chart_path` is present OR assume standard destination usage.
  - *Decision*: Allow `repo_name: null`. If `chart_path` is provided, `deploy` uses it. If not, `deploy` uses `destination/name`.

## Sync Logic
- In `sync.rs`:
  - Check if `repo_name` is `Some`.
  - If ``None``, log "Skipping local chart" and continue.

## Deploy Logic
- In `deploy.rs`:
  - Current logic calculates `dest_path` based on `destination` config.
  - `helm upgrade` command usually takes that path.
  - If `repo_name` is `None` AND `chart_path` is set:
    - Should we override `dest_path` with `chart_path`?
    - `chart_path` in `Chart` struct is currently "path within git repo".
    - For local charts, `chart_path` can mean "local filesystem path".
    - *Action*: In `deploy_chart`, if `repo_name` is None:
      - If `chart_path` is Some, use it as the chart argument logic.
      - Else, use standard `dest_path`.
