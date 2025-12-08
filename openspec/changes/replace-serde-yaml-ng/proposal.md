# Proposal: Replace serde_yaml with serde_yaml_ng

## Goal
Replace the deprecated `serde_yaml` crate with its maintained fork `serde_yaml_ng`. This ensures long-term maintenance and security updates for YAML serialization/deserialization.

## Changes
- **Dependency Update**: Swap `serde_yaml` for `serde_yaml_ng` in `Cargo.toml`.
- **Code Refactoring**: Update imports in `src/config.rs`, `src/lock.rs` (and any other files) to use `serde_yaml_ng`.

## Verification
- **Compilation**: Ensure code compiles without errors.
- **Tests**: Verify all tests pass, preserving existing YAML behavior.
