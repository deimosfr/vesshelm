# Tasks

- [x] Update `DeployStatus` enum in `src/cli/commands/deploy.rs` to include `Ignored` variant
- [x] Update `deploy_chart` function to return `Ok(DeployStatus::Ignored)` when user confirmation is false
- [x] Update `run` function in `src/cli/commands/deploy.rs` to track `ignored_count`
- [x] Update `run` function to print `Ignored` count in the final summary
