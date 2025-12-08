# Design: Chart Deletion

## Workflow

```mermaid
graph TD
    A[User runs vesshelm delete] --> B{Chart Name Provided?}
    B -->|Yes| C[Use specific chart]
    B -->|No| D[Prompt interactive selection]
    D --> C
    C --> E[Check Dependencies]
    E -->|Has Dependents| F[Abort & List Dependents]
    E -->|No Dependents| G[Calculate Impact]
    G --> H[Display Summary]
    H --> I{User Confirms?}
    I -->|No| J[Abort]
    I -->|Yes| K[Delete Chart Directory]
    K --> L[Remove from vesshelm.yaml]
    L --> M[Remove from vesshelm.lock]
    M --> N[Check Repo Usage]
    N -->|Repo Unused| O[Remove Repo from vesshelm.yaml]
    N -->|Repo Used| P[Done]
    O --> P
```

## Data Consistency

*   **Config (`vesshelm.yaml`)**: The source of truth for desired state. Removing the entry stops future syncs/deploys.
*   **Lockfile (`vesshelm.lock`)**: Reflects the current local state. Removing the entry keeps it consistent with `vesshelm.yaml`.
*   **Filesystem**: The actual chart files. Must be removed to reclaim space and avoid confusion.

## Logic Details

*   **Dependency Check**: Reuses `vesshelm::util::dag::get_dependents`. This returns a list of immediate dependents. Since we only delete one chart at a time, checking immediate dependents is sufficient to block the operation.
*   **Repository Cleanup**: Iterate through all *remaining* charts in the config. If `chart.repo_name` matches the deleted chart's repo, and no other chart uses it, mark it for deletion.
