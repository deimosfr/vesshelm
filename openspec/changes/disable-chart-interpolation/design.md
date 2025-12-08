# Design: Disable Chart Interpolation

## Overview
This change introduces a per-chart configuration option to disable Jinja2 variable interpolation in values files.

## Configuration Schema
The `Chart` struct in `src/config.rs` will be updated:

```rust
pub struct Chart {
    // ... existing fields
    #[serde(default)]
    pub no_interpolation: bool,
}
```

## Logic Changes
In `src/cli/commands/deploy.rs`, the `deploy_chart` function currently checks `!variable_context.is_null()` to decide whether to render values files.

We will modify this check to:
```rust
if !variable_context.is_null() && !chart.no_interpolation {
    // proceed with interpolation
}
```

This applies to both:
1. Implicit local `values.yaml` for local charts.
2. Explicitly specified `values_files`.

## Edge Cases
- If `no_interpolation` is true, but no variables are loaded, behavior is the same (no interpolation).
- If `no_interpolation` is false (default), behavior is the same (interpolation if variables exist).
- If `no_interpolation` is true, and values file contains `{{ var }}`, it will be passed literally to Helm. Helm might error if it tries to interpret it, or use it as string, depending on context. This is intended behavior (user control).
