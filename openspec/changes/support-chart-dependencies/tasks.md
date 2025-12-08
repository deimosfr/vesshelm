1.  Implement DAG Logic
    - Create `src/util/dag.rs` (or similar) to handle graph sorting.
    - Implement `sort_charts(charts: &[Chart]) -> Result<Vec<&Chart>>`.
    - Include unit tests for cycles and correct ordering.
2.  Update Deploy Command
    - Modify `src/cli/commands/deploy.rs`.
    - Use variables from `src/util/dag.rs` to order charts before iteration.
3.  Implement Graph Command
    - Create `src/cli/commands/graph.rs`.
    - Use `sort_charts` to get the order.
    - Print the order with dependency info.
    - Register command in `src/cli.rs` (or main.rs/commands.rs).
4.  Verify
    - Run `vesshelm graph` to check output.
    - Run `vesshelm deploy --dry-run` to check order.
