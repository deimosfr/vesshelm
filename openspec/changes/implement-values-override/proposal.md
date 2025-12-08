# Change: Implement Values Override

## Why
Currently, the `values` and `values_files` fields in the `Chart` configuration are ignored during deployment. Users expect these configurations to be applied to the Helm release.

## What Changes
- Modify `src/cli/commands/deploy.rs` to process `values` and `values_files`.
- Convert the `values` list (inline YAML) into a temporary YAML file during deployment.
- Append `-f <path>` arguments to the Helm command for both `values_files` and the generated temporary values file.
- Ensure temporary files are cleaned up (handled automatically by `tempfile` crate).

## Impact
- Affected specs: `chart-deployment`
- Affected code: `src/cli/commands/deploy.rs`
- Breaking: No.
