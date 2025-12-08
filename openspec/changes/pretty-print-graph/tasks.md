## 1. Implementation
- [x] 1.1 Add `ptree` to `Cargo.toml`.
- [x] 1.2 Modify `src/cli/commands/graph.rs`:
  - Implement logic to build a "Dependents Map" (Reverse dependency graph).
  - Use `ptree` to construct and print the visual tree starting from independent charts.
- [x] 1.3 Verify output visually.
- [x] 1.4 Implement coloring of the graph nodes based on tree depth.
- [x] 1.5 Implement configuration validation in `src/cli/commands/graph.rs` before graph generation.
