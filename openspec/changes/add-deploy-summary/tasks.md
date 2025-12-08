# Tasks: Add Deploy Summary

- [x] Initialize `deployed_count` (0), `failed_count` (0), and `skipped_count` (0) in `deploy.rs`.
- [x] Increment `skipped_count` when `no_deploy` is true or user skips interactive confirmation.
- [x] Increment `failed_count` when `deploy_chart` returns an error (caught in loop).
- [x] Increment `deployed_count` when `deploy_chart` succeeds.
- [x] Print summary table at the end of `deploy.rs::run`.
