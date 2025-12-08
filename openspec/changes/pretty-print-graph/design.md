# Design: Pretty Print Graph

## Dependencies
- `ptree = "0.4"` (or latest)

## Implementation
- Load charts and build DAG (using existing `daggy` logic).
- Identify "root" nodes in the dependency graph. In a dependency graph where A depends on B (A -> B), B is the dependency.
- Usually, we want to see "What needs to be deployed first?".
- Flat list shows order.
- Tree view:
  - Option A: Dependency Tree. Root = Independent charts. Children = Charts that depend on them (Reverse dependency).
    - If `my-app` depends on `cilium`.
    - `cilium` (Root)
      - `my-app`
    - This shows "Impact" or "Flow".
  - Option B: Requirement Tree. Root = Top-level apps. Children = Their dependencies.
    - `my-app`
      - `cilium`
    - This shows "What `my-app` needs".
- Given `vesshelm graph` is often about deployment order, Option A seems more aligned with the execution flow (Leafs to Roots in depend-on graph, or Roots to Leafs in depend-by graph).
- **Decision**: We will implement specific logic to visualize the *reverse* dependency graph. Roots are charts with 0 dependencies. Children are charts that depend on the parent.
- **Handling Diamonds**: If C depends on A and B. A and B depend on Base.
  - Base
    - A
      - C
    - B
      - C
  - C appears twice. This is acceptable for a tree view.

## Algorithm
1. Build `daggy::Dag`.
2. Construct a "Reverse" adjacency map (Parent -> Children) where Parent is the dependency.
   - Iterate all charts. For each chart D that depends on P, add D to P's list of dependents.
3. Identify all charts with NO dependencies (Roots).
4. Use `ptree` to build a `StringItem` tree.
   - For each Root, create a tree item.
   - Recursively add children using the Reverse map.
5. Print the tree using `ptree::print_tree`.
