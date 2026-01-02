# Add 'secrets_files' Configuration

## Summary
Add a new configuration field `secrets_files` to `vesshelm.yaml`. This field mirrors `variable_files` but is semantically designated for sensitive data. Files listed here will be loaded and merged into the global variable context.

## Motivation
While `variable_files` can hold any data (including secrets), users often prefer to separate configuration (plain text) from secrets (encrypted). A dedicated `secrets_files` section makes this distinction explicit in the configuration file.

## Goals
- Add `secrets_files` to `vesshelm.yaml`.
- Ensure files in `secrets_files` are loaded and merged into the variable context.
- Ensure `secrets_files` supports SOPS decryption (inherited from the underlying file loader).
- Validate that `secrets_files` paths exist.
### Secrets and Variables Interpolation
Enable variable interpolation within `secrets_files` and `variable_files`. Values in these files should be processed as templates, allowing them to reference previously loaded variables or values within the same context.

## Non-Goals
- Enforcing encryption on `secrets_files` (though it is recommended).
