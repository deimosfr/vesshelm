# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What This Project Is

**Vesshelm** is a GitOps-based Helm chart management tool (Rust, Edition 2024). It manages Helm charts from multiple sources (Helm repos, Git, OCI registries, local files), handles dependencies with topological ordering, and supports AI-powered update analysis.

## Build & Test Commands

```bash
# Build
cargo build               # Debug build
cargo build --release     # Optimized release

# Test
cargo test                # Run all tests
cargo test <test_name>    # Run a single test by name
cargo test -- --nocapture # Show stdout during tests

# Lint & Format
cargo clippy              # Lint
cargo fmt                 # Format

# Via justfile (wraps above with extras)
just fix                  # Fix clippy warnings and format
just test                 # Pre-commit hooks + cargo test + coverage (75% min)
just install              # Build and install locally
```

CI requires Helm and helm-diff plugin for integration tests.

## Architecture

### Module Structure

- **`src/main.rs`** — Async entry point; CLI orchestration with Ctrl+C/SIGTERM signal handling
- **`src/config.rs`** — Strongly-typed config model (`Config`, `Repository`, `Chart`, `Destination`) loaded from `vesshelm.yaml`; validated with the `validator` crate
- **`src/cli/`** — Clap-based CLI; `commands/mod.rs` dispatches to individual command modules
- **`src/engine/sync.rs`** — Core sync engine; concurrent chart fetching with semaphore-based rate limiting
- **`src/clients/`** — External integrations: `helm.rs` (Helm CLI wrapper), `git.rs`, `oci.rs`, `artifacthub.rs`, and `ai/` (Claude, Gemini, OpenAI, Ollama)
- **`src/util/`** — Utilities: DAG dependency ordering (`dag.rs`), Jinja2 templating (`variables.rs`), SOPS encryption (`encryption.rs`), config YAML updates (`config_updater.rs`), UI/progress bars
- **`src/lock.rs`** — `vesshelm.lock` lockfile management

### Key Patterns

1. **Trait-based abstraction**: `HelmClient`, `GitClient`, `Interaction`, and `AIProvider` traits enable mocking in tests (via `mockall`)
2. **Error handling**: `thiserror`-based error enums — root `VesshelmError` in `src/error.rs` unifies `ConfigError`, `ClientError`, `SyncError`, etc.
3. **Async throughout**: Tokio runtime; concurrent syncing uses `tokio::sync::Semaphore`
4. **DAG deployment order**: Charts declare dependencies; `util/dag.rs` does topological sort before deploy/uninstall

### Configuration

`vesshelm.yaml` is the primary config file. Key config types in `src/config.rs`:
- `Repository` — chart sources (type: helm/git/oci)
- `Chart` — individual chart with `repository`, `version`, `destination`, optional `dependencies`, `values`, `secrets_files`, `variable_files`
- `Destination` — local paths for synced charts
- `AIConfig` — AI provider settings for `ai-update` command

Values files support Jinja2 templating (`minijinja`). Secrets files are SOPS-encrypted.

### Testing

Tests live in `tests/` mirroring `src/` structure. The `HelmClient` trait is mocked with `mockall` for unit tests. Integration tests require Helm to be installed.
