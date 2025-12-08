# Refactor Tests Structure

## Goal
Restructure the `tests/` directory to mirror the `src/` directory structure, making it easier to locate tests for specific components and commands.

## Motivation
The current `tests/` directory contains a mix of file naming conventions (`more_sync.rs`, `fail_sync.rs`, `sync_lock_test.rs`) and flat structure which makes it difficult to associate tests with their corresponding source code. Aligning the test structure with the source structure improves maintainability and discoverability.

## Proposed Structure
The `tests/` directory will be reorganized to match `src/`:

```
tests/
  cli/
    commands/
      check_updates.rs  (merges check_updates.rs, fail_check_updates.rs, more_check_updates.rs)
      deploy.rs         (merges deploy.rs, fail_deploy.rs, more_deploy.rs, deploy_config.rs)
      sync.rs           (merges sync.rs, fail_sync.rs, more_sync.rs, sync_config.rs, sync_lock_test.rs, sync_oci.rs)
      init.rs           (merges init.rs)
      graph.rs          (merges graph.rs)
      validate.rs       (merges validate.rs, more_validate.rs)
      uninstall.rs      (merges uninstall.rs)
      version.rs        (merges version.rs)
      mod.rs
    mod.rs
```

## Risks
- Renaming and moving tests might break CI if not careful (e.g., test command arguments).
- Git history for moved files might be slightly harder to trace (though git handles moves well).
