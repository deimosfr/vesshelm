## 1. Implementation
- [x] 1.1 Add `daggy` to `Cargo.toml`.
- [x] 1.2 Rewrite `src/util/dag.rs` to use `daggy::Dag` for graph construction and handling `WouldCycle` errors.
- [x] 1.3 Use `petgraph::algo::toposort` to implement sorting.
- [x] 1.4 Verify implementation with existing tests and `vesshelm graph`.
