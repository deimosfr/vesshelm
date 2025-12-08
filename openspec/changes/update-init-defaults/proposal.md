# Change: Update Init Defaults

## Why
The current defaults in the `init` command do not reflect the newly added standard helm arguments such as `--wait`, `--rollback-on-failure`, and `--create-namespace`. Users have to manually update `vesshelm.yaml` after initialization.

## What Changes
- Update `src/cli/commands/init.rs` to include the new default `helm.args` in the generated `vesshelm.yaml`.

## Impact
- Affected specs: `cli-initialization`
- Affected code: `src/cli/commands/init.rs`
- Breaking: No, only affects new initializations.
