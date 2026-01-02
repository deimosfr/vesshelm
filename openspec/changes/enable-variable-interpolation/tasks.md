- [x] **Core: Configuration**
  - [x] Update `Config` struct in `src/config.rs` to add `variable_files: Option<Vec<String>>`.
  - [x] Add validation for `variable_files` existence.
  - [x] Improve error reporting for config parsing (remove generic context or improve it).
  - [x] Improve validation error formatting (remove `Field '__all__'`).

- [x] **Core: Variable Loading**
  - [x] Create `src/util/variables.rs` (or similar) to handle loading and merging of YAML variable files.
  - [x] Implement `load_variables(paths: &[String]) -> Result<Value>`.

- [x] **Core: Templating**
  - [x] Add `minijinja` dependency.
  - [x] Implement `render_values_file(path: &Path, context: &Value) -> Result<String>`.

- [x] **Integration: Helm Execution**
  - [x] functions in `src/util/helm.rs` or `deploy.rs` to support using rendered files.
  - [x] Update `deploy` command to prepare temp values files before calling Helm.
  - [x] Update `diff` command (if separate) to use temp values files.
  - [x] Implement interpolation for local chart default `values.yaml`.
  - [x] Implement graceful shutdown handler (SIGTERM) to ensure temporary values files are deleted.

- [x] **CLI: Debugging**
  - [ ] (Optional) Add `--keep-temp-values` or similar to inspect rendered files?

- [x] **Tests**
  - [x] Unit tests for variable merging.
  - [x] Unit tests for Jinja rendering.
  - [x] Integration test: mocked Helm call receives temp file with interpolated values.
  - [x] Start adding tests for the new interpolation code (local `values.yaml` logic).
  - [x] Add integration test for error handling (missing variables).
