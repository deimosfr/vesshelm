# Design: Refactor Configuration Structure

## Architecture
The change is primarily a schema refactor. No new logic or behavior is introduced, only the mapping of YAML to Rust structs key names.

## Migration
This is a breaking change. Existing `vesshelm.yaml` files will need to be updated manually by users.
- `helm:` -> `vesshelm:`
- `  args:` -> `  helm_args:`
