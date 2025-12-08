# Goal
Improve error reporting for OCI `check-updates` failures.

# Context
The user is encountering `Failed to fetch OCI version: helm show chart failed` when running `check-updates` against an OCI registry. The generic error message matches `OciError::CommandFailed`, which currently swallows the `stderr` from the Helm command. To debug the root cause (e.g. auth issues, missing tags, invalid URL), we must expose the underlying error.

# Changes
1.  **Modify `check-updates` commands**: Update `src/cli/commands/check_updates.rs`:
    *   Change `OciError::CommandFailed` to `OciError::CommandFailed(String)`.
    *   Capture `stderr` when `helm show chart` fails and propagate it.
