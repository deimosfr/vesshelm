# Change: Add Local Chart Deployment

## Why
Currently, Vesshelm can download charts but cannot deploy them. Users need to rely on external tools or manual `helm` commands. This change enables Vesshelm to deploy charts directly using a local `helm` binary, bridging the gap with tools like Helmfile.

## What Changes
- Add `helm` configuration section to `vesshelm.yaml`.
- Update `charts` configuration to support deployment-specific fields (`helm_args_append`, `helm_args_override`, `destination_override`).
- Add new `deploy` command to the CLI.
- Implement variable interpolation for helm arguments.

## Impact
- Affected specs: `chart-deployment` (new capability)
- Affected code: `vesshelm.yaml` parsing, `main.rs`, `commands/mod.rs`

