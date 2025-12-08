## 1. Implementation
- [x] 1.1 Refactor `src/config.rs` to expose a public validation function (usable by both `validate` command and `sync`).
- [x] 1.2 Update `src/cli/commands/sync.rs` to call validation before processing.
- [x] 1.3 Refactor `sync` logic to rely on validated state (remove redundant checks like "repo exists" if validation guarantees it).
- [x] 1.4 Verify `sync` fails gracefully on invalid config (via new or existing tests).
