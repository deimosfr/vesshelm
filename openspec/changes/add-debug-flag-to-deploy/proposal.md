# Proposal: Add Debug Flag to Deploy Command

## Summary
Add a `--debug` argument to the `deploy` command. When this flag is used, it should append `--debug` to the underlying `helm upgrade` command.

## Motivation
Users often need to debug Helm deployments when they fail. Helm supports a `--debug` flag that enables verbose output, which is helpful for troubleshooting. Vesshelm should expose this capability directly in its `deploy` command to facilitate debugging.

## Solution Design
- Update `DeployArgs` struct in `src/cli/commands/mod.rs` to include a `debug` boolean field.
- Update `DeployOptions` struct in `src/cli/commands/deploy.rs` to include `debug` field.
- In `deploy_chart` function in `src/cli/commands/deploy.rs`, append `--debug` to the Helm command arguments if the flag is present.
- Ensure the debug flag is passed correctly from CLI args to the deployment logic.

## Detailed Design
No complex architectural changes are required. The flag will be propagated from the CLI arguments down to the `deploy_chart` function where the helm command is constructed.

### Alternatives Considered
- Using `helm_args_append` in config: This works but is persistent and per-chart. A CLI flag is better for ad-hoc debugging.
- Using an environment variable: Less discoverable than a CLI flag.

## Risks
None identified. This is a purely additive change to the CLI interface and passes a standard flag to Helm.
