# Improve Error Reporting

## Why
When dependency resolution or configuration validation fails, the error message often hides the root cause (e.g., "Failed to resolve chart dependencies" without specifically explicit details). Users are left without knowing what to fix.

## What Changes
- Update the global error formatter in `src/util/error.rs` to display the full error chain for general errors.
- This ensures that if `anyhow` context is added (like "Failed to resolve..."), the underlying cause ("Chart A depends on B...") is also printed.

This does not require changing `serde` parsing for line numbers immediately, as most logical errors (dependencies) already contain precise identifiers (chart names). By revealing them, we solve the ambiguity.
