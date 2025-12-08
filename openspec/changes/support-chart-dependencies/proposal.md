# Change: Support Chart Dependencies

## Why
Currently, Vesshelm deploys charts sequentially in the order they are defined in `vesshelm.yaml`. This is insufficient for managing complex stacks where some services (e.g., Cert Manager, Cilium) must be fully ready before others start. Users need to explicitly define dependencies so that Vesshelm can compute the correct deployment order (DAG) and wait for prerequisites.

## What Changes
- Add support for a `depends` field in chart configuration.
- Implement topological sorting to determine deployment order.
- Implement a `vesshelm deploy` logic update to respect this order.
- Add a new command `vesshelm graph` to visualize the dependency tree.

## Impact
- Affected specs: `dependency-management`, `cli-commands`
- Affected code: `src/cli/commands/deploy.rs`, `src/config.rs` (logic only, struct exists), new `src/cli/commands/graph.rs`
- Breaking: No.
