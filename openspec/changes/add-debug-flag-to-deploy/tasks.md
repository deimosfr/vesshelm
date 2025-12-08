# Tasks

1. [x] Update `DeployArgs` struct in `src/cli/commands/mod.rs` to add `debug` field.
2. [x] Update `DeployOptions` struct in `src/cli/commands/deploy.rs` to add `debug` field.
3. [x] Update `run` function in `src/cli/commands/deploy.rs` to populate `DeployOptions` with debug flag from args.
4. [x] Update `deploy_chart` function in `src/cli/commands/deploy.rs` to append `--debug` to helm arguments if flag is set.
5. [x] Add a test case in `tests/cli/commands/deploy.rs` to verify that `--debug` is passed to the mock helm command.
