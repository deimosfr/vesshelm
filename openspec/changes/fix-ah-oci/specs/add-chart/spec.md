# Spec: Add Chart Delta

## MODIFIED Requirements

### Requirement: Artifact Hub Parsing
The `add` command MUST parse the provided Artifact Hub URL to extract chart details AND correctly identify the repository type (Helm or OCI).

#### Scenario: Artifact Hub OCI Source
Given the user provides an Artifact Hub URL for a chart hosted in an OCI registry (e.g. `oci://...`)
Then the system MUST detect `RepoType::Oci`
And the system MUST configure the repository with the correct OCI URL.

#### Scenario: Unit Tests for Mapping
Given the `ArtifactHubSource` implementation
Then there MUST be unit tests verifying that `oci://` URLs result in `RepoType::Oci` details.
