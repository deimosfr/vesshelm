# Change: Add Helm Diff Support

## Why
Users want to inspect changes before deploying charts to avoid unexpected modifications. The `helm-diff` plugin is a standard tool for this. Integrating it into Vesshelm allows for safer deployments, supporting both a "dry-run" mode and an automatic "diff-before-deploy" workflow.

## What Changes
- Add `diff_enabled` (bool) and `diff_args` (string) to `HelmConfig`.
- Add `--dry-run` flag to `deploy` command.
- Update `deploy` command logic.
- Update `init` command to generate default diff configuration.
    - If `--dry-run` is set, ONLY run `helm diff`.
    - If `diff_enabled` is true (and not dry-run), run `helm diff` BEFORE `helm upgrade`.
- Add `helm-diff` plugin check (optional but good practice).

## Impact
- Affected specs: `chart-deployment`
- Affected code: `src/config.rs`, `src/cli/commands/deploy.rs`
