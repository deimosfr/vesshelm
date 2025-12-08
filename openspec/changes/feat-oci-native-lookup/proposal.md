# Goal
Implement native OCI tag listing to support `check-updates` for OCI charts that lack a `latest` tag.

# Context
Helm's `show chart` command fails with "unable to locate any tags" when an OCI repository does not have a `latest` tag, preventing `vesshelm` from checking for updates. To resolve this, we must bypass Helm and query the OCI registry directly to list and sort available tags.

# Changes
1.  **New Client**: Create `src/clients/oci.rs` with `OciClient`.
    *   Implement `get_tags(repo_url, chart_name)` using `reqwest`.
    *   Handle OCI Token Authentication (Www-Authenticate challenge), supporting public GHCR/ECR access.
2.  **Integrate**: Modify `src/cli/commands/check_updates.rs`.
    *   If `helm show chart` fails, fallback to `OciClient::get_tags`.
    *   Parse semver from tags and find the greatest version.
