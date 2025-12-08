# Design: Secrets Files Configuration

## Configuration Change
We will add `secrets_files` to the `Config` struct in `src/config.rs`.

```rust
pub struct Config {
    // ...
    pub variable_files: Option<Vec<String>>,
    pub secrets_files: Option<Vec<String>>, // [NEW]
}
```

## Validation
We will add validation logic in `validate_config` to ensure all files listed in `secrets_files` exist, similar to `variable_files` validation.

## Loading Logic
In `src/cli/commands/deploy.rs` (and other commands loading variables), we will:
1.  Load `variable_files`.
2.  Load `secrets_files`.
3.  Merge them.

Since `load_variables` takes a list of paths, we can potentially concatenate the lists (`variable_files` + `secrets_files`) and load them in one go, maintaining precedence (secrets usually override variables? Or variables override secrets? Usually secrets should probably take precedence or just rely on order).

We will assume `variable_files` comes first, then `secrets_files` overrides it if there are collisions, effectively:
`context = merge(variables, secrets)`

## Encryption
Since we updated `load_variables` to support SOPS transparently, `secrets_files` will automatically support SOPS encryption if the file has the sops metadata.

## Interpolation Strategy
Variable files (including secrets) should now be treated as templates.
When loading multiple variable files:
1. Load/Decrypt files in order.
2. Maintain a cumulative variable context.
3. For each file, render it using the current context BEFORE parsing it as YAML.
4. Merge the result into the context.

This allows later files to reference values from earlier files.
