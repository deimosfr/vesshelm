# Design: Automatic SOPS Decryption

## Strategy
We will introduce a transparent decryption layer in the file reading utilities.

### Detection
We will identify SOPS files by reading the file content and checking for the presence of the `sops` key at the root level of the YAML/JSON structure. This is safer than relying on file extensions, as users often keep `.yaml` for editor support.

Naive check: `content.contains("sops:") && content.contains("mac:")` (to be refined during implementation, possibly using `serde_yaml` to check for root "sops" key if performance allows, or a regex).

### Decryption
We will utilize the installed `sops` binary.
Command: `sops --decrypt --output-type yaml <file_path>`
This ensures we support all formats `sops` supports and keys configurated in the environment.

### Integration Points
1.  **`src/util/variables.rs`**:
    -   `load_variables`: Currently iterates paths and reads them. Will utilize `read_possibly_encrypted_file`.
    -   `render_values_file`: Currently reads file and renders with MiniJinja. Will utilize `read_possibly_encrypted_file` *before* rendering. This allows secrets to be used in templates or templates to include secrets.

### Error Handling
-   If `sops` binary is missing but a file looks encrypted: Fail with a clear error "SOPS encrypted file detected but 'sops' binary not found in PATH".
-   If decryption fails: Propagate the sops error output.

### Dependencies
-   No new Rust crate dependencies.
-   Runtime dependency: `sops` CLI tool.
