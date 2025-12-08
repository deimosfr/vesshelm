# Design: Sync Optimization with Lockfile

## Lockfile Format
The `vesshelm.lock` file will be a YAML file storing a list of synced charts.

```yaml
charts:
  - name: adguard-home
    repo_name: rm3l
    version: 0.21.0
```

```rust
#[derive(Debug, Deserialize, Serialize)]
struct Lockfile {
    charts: Vec<SyncedChart>
}

#[derive(Debug, Deserialize, Serialize)]
struct SyncedChart {
    name: String,
    repo_name: String, // To differentiate same chart name from different repos (if needed, or just standard name)
    version: String
}
```

## Sync Logic Flow

1. **Initialization**:
   - Parse `vesshelm.yaml`.
   - Try to parse `vesshelm.lock`. If missing or invalid, start with empty list.

2. **Iteration**:
   - Loop through charts in `vesshelm.yaml`.
   - Resolve `repo_name` and `version`.
   - **Check Lock**:
     - Find entry in `Lockfile` with matching `name` and `repo_name`.
     - If found AND `entry.version == chart.version`:
       - If `--ignore-skip` is NOT set:
         - Print "Skipped (up to date)".
         - Continue to next chart.

3. **Execution**:
   - Perform pull/copy as usual.
   - If successful, update/add entry in `Lockfile` in memory.

4. **Finalization**:
   - Write `Lockfile` to `vesshelm.lock` on disk.

## CLI Changes
- `vesshelm sync` adds flag `--ignore-skip` to force sync even if locked.
