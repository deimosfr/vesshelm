## 1. Implementation
- [x] 1.1 Add `validator` and `validator_derive` to `Cargo.toml`
- [x] 1.2 Implement `Validate` trait for `Config`, `Repository`, `Chart`, `Destination` in `src/config.rs`
- [x] 1.3 Implement `src/cli/commands/validate.rs` to parse and run `.validate()`
- [x] 1.4 Register `validate` subcommand in `src/cli/commands/mod.rs` and `src/main.rs`
- [x] 1.5 Add integration test for `validate` command (valid and invalid configs)
- [x] 1.6 Implement duplicate chart validation (name + namespace combination) with parameters for error formatting
- [x] 1.7 Implement user-friendly error printing in `src/cli/commands/validate.rs`
