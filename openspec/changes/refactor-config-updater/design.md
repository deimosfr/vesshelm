# Design: Centralized Config Updater

## Problem
The configuration update logic is split between:
1. `src/cli/commands/add/config_updater.rs` (Appends new repos/charts)
2. `src/cli/commands/check_updates.rs` (Updates existing chart versions in place using regex)

## Solution
Consolidate both behaviors into a single `ConfigUpdater` struct in `src/util/config_updater.rs`.

### Architecture

#### `src/util/config_updater.rs`

```rust
pub struct ConfigUpdater;

impl ConfigUpdater {
    // Existing functionality from add command
    pub fn append_repo_chart(config_path: &Path, repo: Option<Repository>, chart: ChartConfig) -> Result<()> { ... }
    
    // Internal helper extracted from add/config_updater.rs
    fn add_repository(content: &mut String, r: &Repository) { ... }
    fn add_chart(content: &mut String, chart: &ChartConfig) { ... }

    // New functionality moved from check_updates.rs
    // Adapting find_and_replace_version to be a method here
    pub fn update_chart_version(config_path: &Path, chart_name: &str, new_version: &str) -> Result<()> {
         // Read, replace, write
    }
    
    // Low-level helper for in-memory string manipulation (for testing)
    pub fn replace_chart_version_in_text(content: &mut String, chart_name: &str, new_version: &str) -> Result<()> {
        // Logic from check_updates.rs::find_and_replace_version
    }
}
```

### Module Changes
- `src/cli/commands/add/config_updater.rs` -> Deleted.
- `src/cli/commands/add/mod.rs` -> Imports `vesshelm::util::config_updater::ConfigUpdater`.
- `src/cli/commands/check_updates.rs` -> Removes local update logic, imports `ConfigUpdater`.
- `src/util/mod.rs` -> Exports `config_updater`.

## Trade-offs
- **Regex vs AST**: We continue to use Regex/string manipulation to preserve comments and structure ("smart update") rather than a full YAML parser/serializer which would strip them. This is consistent with current behavior.
