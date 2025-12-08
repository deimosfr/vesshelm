# Tasks

- [x] Update `VesshelmConfig` struct to include `helm` section with `args`.
- [x] Update `Chart` struct to support new fields: `comment`, `helm_args_append`, `helm_args_override`, `destination_override`, `depends`, `values`.
- [x] Implement `deploy` command scaffolding in `main.rs` and `commands/mod.rs`.
- [x] Implement variable interpolation logic for `helm` args strings.
- [x] Implement `deploy` logic:
    - [x] Resolve dependencies for ordering (if `depends` is used).
    - [x] Iterate through charts.
    - [x] Construct final command line arguments.
    - [x] Execute `helm` command for each chart.
- [x] Add `validate` checks for new configuration fields.
- [x] Verify `deploy` command with existing and mocked charts.
