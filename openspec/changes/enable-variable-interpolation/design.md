# Design: Variable Interpolation Architecture

## core-concepts

### Variable Loading
-   **Source**: `variable_files` list in `vesshelm.yaml`.
-   **Format**: YAML files containing key-value pairs.
-   **Merger**: Files are loaded in order. Later files override earlier ones. Result is a single global context object.

### Interpolation Engine
-   **Engine**: `minijinja` (Rust Jinja2 implementation) is lightweight and sufficient.
-   **Context**: The merged global variables.
-   **Target**: The files specified in `chart.values_files`.
-   **Syntax**: Standard Jinja2 `{{ variable }}`.

### Execution Flow
1.  **Read Config**: Parse `vesshelm.yaml`.
2.  **Load Variables**: Read and merge `variable_files`.
3.  **For Each Chart**:
    *   Identify `values_files` overrides.
    *   For each override file:
        *   Read content.
        *   Render content using `minijinja` with global context.
        *   Write rendered content to a named temporary file (e.g., in a temp dir associated with the run).
    *   Construct Helm command arguments using paths to temporary files instead of original paths.
    *   Execute Helm (diff/install/upgrade).
4.  **Cleanup**: temporary files are cleaned up after execution (automatically via `tempfile` crate or manual cleanup).

## Trade-offs
-   **Complexity**: Adds a rendering step and temporary file management.
-   **Debuggability**: Helm will report errors in temporary files. We might need options to inspect generated values (e.g., `--debug` keeps temp files or prints them).
-   **Performance**: Negligible overhead for rendering small YAML files.

## Alternatives
-   **Helm --set**: Generating massive `--set` strings. Hard to manage complex data structures.
-   **Post-processing**: Using `envsubst`. Less powerful than Jinja (no loops, logic).
