# Design: Improved Error Reporting

## Current Implementation
In `src/cli/commands/deploy.rs`, errors during chart deployment are caught and printed using:
```rust
tracker.println(&format!(
    " {} {} {}: {}",
    style("[Fail]").red(),
    "✗",
    chart.name,
    e
));
```
The `{}` formatter for `anyhow::Error` only prints the outermost context (e.g., "Failed to render values file ..."). The underlying cause (from `minijinja` or `serde_yaml_ng`) is hidden in the error source chain.

## Proposed Change
Modify the error logging to use the alternate formatter `{:#}`, which prints the error and its causal chain. Alternatively, iterate over the chain explicitly if finer control is needed.
For `anyhow`, `{:#}` is the standard way to show the chain.

Example output change:
**Before:**
`[Fail] ✗ chart-name: Failed to render values file "./overrides/freshrss.yaml"`

**After:**
`[Fail] ✗ chart-name: Failed to render values file "./overrides/freshrss.yaml": undefined value 'foo'`

## Alternatives Considered
- **Explicit Chain Iteration**: We could write a helper to iterate `e.chain()` and format it nicely. This might offer better UI than the default `{:#}` which might be too verbose or raw. Given `vesshelm`'s polished UI, a custom formatted chain might be better.
- **Minijinja Error Mapping**: We could map `minijinja` errors to a custom error type, but `anyhow` is sufficient if displayed correctly.

## Decision
Use `{:#}` for immediate improvement. If the output is too messy, refine with a custom chain printer.
