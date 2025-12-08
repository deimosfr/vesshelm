# Change: Add Validate Command

## Why
Users need a way to verify their `vesshelm.yaml` configuration is correct without running a full sync. The tool currently errors out deeply in execution if the config is invalid. Validating it explicitly prevents runtime errors and improves the user experience.

## What Changes
- Add `validate` command to the CLI.
- Implement strict validation logic for `Config` structs:
    - `repositories`: unique names, valid URLs.
    - `charts`: valid names, existing repositories, unique names/paths, valid version strings.
    - `destinations`: unique names, valid paths.
- Use `validator` crate (or custom `impl Validate`) and serde attributes to enforce constraints.
- Structure code to separate validation logic from configuration definition if possible, or assume traits on Config structs.

## Impact
- **Dependencies**: Add `validator` (and `validator_derive`) to `Cargo.toml`.
- **Code**: `src/config.rs` (add validation derives/impls), `src/cli/commands/validate.rs` (new command), `src/cli/commands/mod.rs` (register command).
