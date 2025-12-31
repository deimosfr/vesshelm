<!-- id: fix-ah-oci -->
# Fix Artifact Hub OCI Tasks

- [x] Refactor Source Logic <!-- id: 1 -->
    - [x] Extract `map_package_to_details` function in `src/cli/commands/add/source.rs`
    - [x] Ensure `oci://` detection logic is robust
- [x] Add Unit Tests <!-- id: 2 -->
    - [x] Test mapping logic for Helm package
    - [x] Test mapping logic for OCI package
- [x] Verification <!-- id: 3 -->
    - [x] Run `cargo test`
    - [x] Manual verification (if possible with known OCI chart)
