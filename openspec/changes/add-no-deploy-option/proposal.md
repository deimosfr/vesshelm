# Proposal: Add `no_deploy` Configuration Option

## Context
Users currently have `no_sync` to skip syncing specific charts. A similar `no_deploy` option is requested to allow syncing charts (e.g., for local inspection or dependency management) without actually deploying them to the cluster. This is useful for charts that are managed manually or are temporarily disabled.

## Goal
Add an optional `no_deploy` boolean field to the `Chart` configuration in `vesshelm.yaml`.
- Default: `false`.
- If `true`: `vesshelm deploy` should skip this chart with a visual indicator (e.g. "[SKIP] (no_deploy=true)").
- `vesshelm sync` should still sync the chart unless `no_sync` is also true.

## Changes

### Configuration
- Update `src/config.rs`: Add `no_deploy: Option<bool>` to `Chart` struct (defaulting to false).

### Deploy Logic
- Update `src/cli/commands/deploy.rs`:
  - Check `chart.no_deploy` inside the deployment loop.
  - If true, print a skip message and `continue`.

## Risk
- Low risk. Purely additive configuration.
