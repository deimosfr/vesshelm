- [x] Update Configuration <!-- id: 1 -->
  - Add `release_name` to `Chart` struct in `src/config.rs`.
  - Add `release_name()` helper method to `Chart`.
- [x] Update Deployment Logic <!-- id: 2 -->
  - Modify `src/cli/commands/deploy.rs`:
    *   Update `interpolate_variables` to support `{{ release_name }}` and `{{ chart_name }}`.
    *   Update `{{ name }}` to resolve to `release_name`.
    *   Ensure path construction uses `chart_name`.
- [x] Update Uninstall Logic <!-- id: 3 -->
  - Modify `src/cli/commands/uninstall.rs` to use `release_name`.
- [x] Verification <!-- id: 4 -->
  - Add unit tests for `interpolate_variables` with `release_name`.
  - Manual verification with `dry-run`.
