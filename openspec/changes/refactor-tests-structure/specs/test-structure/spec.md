# Test Structure Specs

## MODIFIED Requirements

#### Test Organization
- **Scenario:** Developer looks for tests
  - **Given** a source file `src/cli/commands/sync.rs`
  - **When** the developer looks into `tests/`
  - **Then** they should find `tests/cli/commands/sync.rs` containing the relevant integration tests.

- **Scenario:** Running specific tests
  - **Given** the new structure
  - **When** running `cargo test --test cli`
  - **Then** it should run all CLI integration tests. (Note: this depends on whether we make one test crate or modules. Proposal assumes modules).
  - *Correction*: If we move to `tests/cli/commands/sync.rs`, we need `tests/cli.rs` (or similar) to make it a test crate, or each file in `tests/` is a crate.
  - *Refinement*: To match `src` exactly where `src/cli/commands/sync.rs` exists, `tests` should probably use a library layout if possible, or just modules under a main test entry point.
  - *Decision*: We will use `tests/cli.rs` as the entry point which declares `mod commands;` and `tests/cli/commands/mod.rs` declares the command modules. This way `cargo test --test cli` runs all CLI tests.

## ADDED Requirements

#### Code Coherence
- The `tests/` directory structure MUST mirror the `src/` directory structure for high-level components (CLI, Engine, etc.).
