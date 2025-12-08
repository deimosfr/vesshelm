# Support SOPS Encryption for Variables and Overrides

## Summary
Add automatic transparent decryption of SOPS-encrypted files when loading variables and chart value overrides. This allows users to store sensitive data (secrets) securely in git while using them seamlessly in `vesshelm`.

## Motivation
Currently, `vesshelm` reads variable files and value overrides as plain text. Users have to handle secret decryption outside of the tool or avoid committing secrets. `sops` is a standard tool for this. Integrating it directly simplifies the workflow.

## Goals
- Automatically detect SOPS-encrypted files.
- Decrypt them in-memory using the `sops` binary.
- Support both variable files (used for global substitution) and chart values/override files.
- Maintain existing templating capabilities (decrypt -> render -> parse).

## Non-Goals
- Editing SOPS files via `vesshelm` (users should use `sops` CLI).
- Managing PGP/AGE keys (users should configure their environment).
- Re-implementing SOPS crypto in Rust (we will shell out to `sops`).
