## 1. Implementation
- [x] 1.1 Implement helper `merge_values(values: &[Value]) -> Result<String>` in `src/util/helm.rs` (or `deploy.rs`) to flatten the values list into a valid YAML map string.
- [x] 1.2 Modify `deploy_chart` in `src/cli/commands/deploy.rs`:
  - Handle `chart.values_files`: append `-f` for each.
  - Handle `chart.values`: call helper, write result to `NamedTempFile`, append `-f` with temp path.
  - Ensure `temp_file` variable stays in scope during execution.
- [x] 1.3 Verify using `vesshelm deploy --dry-run` and checking generated args (and content if possible, or trust dry-run success).
