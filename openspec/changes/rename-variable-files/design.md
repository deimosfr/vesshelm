# Design: Rename Variable Files

## Code Changes
1.  **Config Struct**:
    ```rust
    // src/config.rs
    #[serde(alias = "variable_files")]
    pub variables_files: Option<Vec<String>>,
    ```
2.  **Validation**: Update `validate_config` to check `variables_files`.
3.  **Usage**: Update `src/cli/commands/deploy.rs` `load_variables` call site to use `config.variables_files`.

## Backward Compatibility
The `serde` alias ensures that existing `vesshelm.yaml` files using `variable_files` continue to parse correctly into the `variables_files` struct field.
