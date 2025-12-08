# Change: Disable Chart Interpolation

## Why
Currently, Vesshelm attempts to interpolate variables in values files using Jinja2 if a variable context is loaded. This can cause issues for charts that use syntax resembling Jinja2 templates (e.g., `{{ ... }}`) in their values files, or if the user simply wishes to disable this behavior for specific charts.

## What Changes
- Add `no_interpolation` boolean field to the `Chart`
- `src/config.rs`: Add field to `Chart` struct.
- `src/cli/commands/deploy.rs`: Update `deploy_chart` to check flag.
- `README.md`: Update Configuration Reference to include `no_interpolation`.

### Impact

- Charts producing values that conflict with Jinja2 syntax (e.g., embedding other Go templates) can now be deployed without render errors.
- **Documentation**: The `no_interpolation` field will be documented in the `README.md` under "Chart Options".
cessing for values files, passing them directly to Helm.

## Impact
- Affected specs: configuration
- Affected code: `src/config.rs`, `src/cli/commands/deploy.rs`
