# Proposal: Enable Variable Interpolation in Values Files

## Summary
Add support for determining common variables in `vesshelm.yaml` (via `variable_files`) and using them to interpolate Jinja2 placeholders within chart values override files. This enables dynamic value generation and reuse of common configuration across different environments or charts.

## Why
Currently, values files are static. Users often need to share common values (like region, environment, global domains) across multiple charts or override specific values based on some global configuration without duplicating data.
By adding Jinja2 support, users can define variables once and reference them in multiple `values-override.yaml` files.

## Solution Overview
1.  **Configuration**: Add `variable_files` field to the root `Config` in `vesshelm.yaml`.
2.  **Loading**: Load and merge variables from these files.
3.  **Interpolation**: When processing charts (diff/deploy/sync), render the configured `values_files` using the loaded variables as context.
4.  **Execution**: Write rendered values to temporary files and pass them to the Helm command.

## Key Changes
-   Update `Config` struct to include `variable_files`.
-   Implement variable loading and merging logic.
-   Integrate a Jinja2 rendering engine (e.g., `minijinja`).
-   Update Helm command execution to use rendered temporary values files.
