# Goal
Allow users to specify a custom `release_name` for a chart deployment, decoupling it from the chart's directory or logical name.

# Context
Currently, `vesshelm` uses the `name` field in `vesshelm.yaml` for both:
1.  The logical identifier of the chart (and its default directory name).
2.  The Helm release name used during `helm install/upgrade` and `helm uninstall`.

Users may want to deploy the same chart multiple times with different release names, or simply have a release name that differs from the repo/chart name.

# Changes
1.  **Configuration**: Add optional `release_name` field to `Chart` struct in `src/config.rs`.
2.  **Deployment**:
    *   Update `src/cli/commands/deploy.rs` to use `release_name` (defaulting to `name`) when constructing Helm commands.
    *   Update variable interpolation:
        *   `{{ name }}` -> Release Name (for `helm upgrade {{ name }} ...`).
        *   `{{ release_name }}` -> Release Name (explicit).
        *   `{{ chart_name }}` -> Chart Name (original `name` field, useful for paths).
        *   **Crucial**: Preserve path construction logic `{{ destination }}/{{ name }}` to use the Chart Name/Folder.
3.  **Uninstallation**:
    *   Update `src/cli/commands/uninstall.rs` to uninstall by `release_name`.
