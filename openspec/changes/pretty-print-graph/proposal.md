# Change: Pretty Print Graph

## Why
The current `vesshelm graph` command outputs a flat list of charts in deployment order. While functional, it is difficult to visualize the actual dependency structure. Users requested a tree-like or pretty-printed output to better understand chart relationships.

## What Changes
- Add `ptree` dependency.
- Modify `vesshelm graph` to output a tree structure of dependencies.
- Iterate through independent charts (roots) and recursively print their dependents.
- Note: Since DAGs can have shared dependencies, a tree view might duplicate nodes or we might need a custom renderer. For simplicity, we will use `ptree` and potentially show shared subtrees multiple times or use references. Alternatively, we can just show the deployment order with improved formatting, but "pretty print graph" strongly implies structure.
- *Refinement*: We will attempt to build a tree where "roots" are charts with no dependencies, and children are charts that depend on them. Wait, `depends` field is "I depend on X". So X is the parent. So it's a dependency tree. `ptree` works well here.

## Impact
- Affected specs: `cli-commands` (modifying `cli-graph-command` spec).
- Affected code: `src/cli/commands/graph.rs`, `Cargo.toml`.
- Breaking: No (visual change only).
