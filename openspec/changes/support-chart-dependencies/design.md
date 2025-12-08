# Design: DAG Deployment & Visualization

## Architecture
- **Dependency Graph**:
  - The `Chart` struct already contains a `depends: Option<Vec<String>>` field.
  - We will implement a Directed Acyclic Graph (DAG) logic to process this.
  - Verification of dependencies: Ensure all referenced dependencies exist in the configuration.
  - Cycle Detection: Use standard DFS or Kahn's algorithm to detect cycles (e.g., A -> B -> A).

## Deployment Logic
1.  **Graph Construction**:
    - Iterate over `config.charts`.
    - Create nodes for each chart.
    - Add edges for each dependency listed in `depends`.
2.  **Ordering**:
    - Perform a topological sort.
    - If a cycle is detected, fail immediately with a descriptive error.
    - If valid, produce a list of charts in execution order.
3.  **Execution**:
    - The `deploy` command will use this sorted list instead of the raw `config.charts` vector.
    - Charts will be deployed sequentially in this order.

## CLI 'Graph' Command
- **Command**: `vesshelm graph`
- **Output**:
  - Prints the charts in Topologically Sorted order.
  - Optionally shows the dependencies for each.
  - Example output:
    ```
    1. cilium
    2. cert-manager (depends on: cilium)
    3. my-app (depends on: cert-manager)
    ```
