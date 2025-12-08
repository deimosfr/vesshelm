# Change: Use Daggy Library

## Why
The current DAG implementation uses a custom implementation of Kahn's algorithm. While functional, it is verbose and reinvents the wheel. Using a specialized library like `daggy` (built on `petgraph`) provides a robust, proven graph data structure, simplifies cycle detection (preventing specific edges at insertion time), and opens the door for advanced graph features (like DOT export or visualization) in the future.

## What Changes
- Add `daggy` (and potentially `petgraph`) to `Cargo.toml`.
- Refactor `src/util/dag.rs` to use `daggy::Dag` for graph construction and sorting.
- Use `daggy`'s cycle detection mechanisms instead of manual Kahn's algorithm cycle check.
- Ensure `vesshelm graph` and `deploy` commands continue to work with the new backend.

## Impact
- Affected specs: `dependency-management` (implementation detail mostly, but reinforces cycle detection requirement).
- Affected code: `src/util/dag.rs`, `Cargo.toml`.
- Breaking: No.
