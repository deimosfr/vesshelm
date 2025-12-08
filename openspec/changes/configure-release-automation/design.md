# Design: Release Automation

## CI Pipeline (Pull Requests)
- **Triggers**: On pull request to any branch.
- **Checks**:
    - `cargo build` (no errors)
    - `cargo build` (no warnings) -> `RUSTFLAGS="-D warnings"`
    - `cargo test` (no errors, no warnings)
    - Coverage check (tarpaulin or similar) >= 60%
    - `cargo audit` (allow failure)

## CD Pipeline (Releases)
- **Triggers**: On push of tags (v*).
- **Checks**:
    - Verify `Cargo.toml` version matches the tag.
- **Actions**:
    - Run `goreleaser release` to build and publish artifacts.

## Main Branch
- No specific actions triggered on push to `main` (as requested).
