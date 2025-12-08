# Fix Artifact Hub OCI Support

## Goal
Ensure `vesshelm add` correctly identifies and configures charts from Artifact Hub that are hosted in OCI registries.

## Context
Artifact Hub listings can point to standard Helm repositories or OCI registries. Currently, the `add` command may incorrectly default to `RepoType::Helm` for OCI URLs, or fail to produce a valid configuration that links the chart to its repository.

## Changes
1.  **Logic Fix**: Update `ArtifactHubSource` to detect `oci://` schemes in the repository URL and set `RepoType::Oci`.
2.  **Testability**: Refactor `ArtifactHubSource` to separate the mapping logic (Package -> ChartDetails) from the network/interactive logic, allowing for unit tests.
3.  **Testing**: Add unit tests to verify:
    *   Artifact Hub package with `https://` URL -> `RepoType::Helm`
    *   Artifact Hub package with `oci://` URL -> `RepoType::Oci`
