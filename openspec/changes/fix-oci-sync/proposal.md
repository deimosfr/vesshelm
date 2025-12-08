# Fix OCI Sync URL Construction

## Goal
Make OCI URL construction during `sync` smarter to support standard OCI artifacts and registry roots, preventing errors when full artifact URLs are used in configuration.

## Context
When syncing Helm charts from OCI registries, `vesshelm` currently naively concatenates the repository URL and the chart name with a slash. This causes errors (likely `could not load config with mediatype...`) when the user provides the full artifact URL in the `repositories` configuration, resulting in a duplicated path segment.

## Changes
1.  **Refactor `HelmClient::pull`**: Update signature to accept a single `chart_ref` string.
    *   Parse `repo.url`; check if the last path segment matches `chart.name`.
    *   If `repo.url` ends with `/chart.name` (path segment match), use `repo.url`.
    *   Otherwise, append `chart.name`.
    *   Ensure `oci://` prefix.
