# Design: Daggy Integration

## Architecture
- **Dependency**: `daggy = "0.9"` (or latest).
- **Graph Construction**:
  - `src/util/dag.rs` will create a `daggy::Dag<Chart, ()>`.
  - Iterate charts to add nodes (NodeIndex).
  - Iterate `depends` to add edges. `daggy` returns a `Result` on adding an edge `update_edge(a, b, weight)` or `add_edge`? `daggy` has `add_edge(parent, child, weight) -> Result<EdgeIndex, WouldCycle>`.
  - Check for `WouldCycle` error immediately during construction.

## Sorting
- **Topological Sort**:
  - `petgraph` (which `daggy` uses) provides `petgraph::algo::toposort`.
  - We can convert/access the underlying graph or use `daggy` specific methods if available.
  - `daggy` guarantees acyclicity, so `toposort` should never fail if construction succeeded.
  - Return `Vec<&Chart>` in sorted order.
