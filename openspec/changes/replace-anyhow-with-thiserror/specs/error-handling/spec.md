## ADDED Requirements

### Requirement: Typed Error Handling
All internal modules and library code MUST use typed errors (e.g., via `thiserror`) instead of opaque `anyhow::Error`. This ensures that errors are explicitly defined and handleable.

#### Scenario: Library Error
Given a function in `src/clients/`
When it fails
Then it should return a specific error enum variant (e.g., `ClientError::Network`)
And not a generic `anyhow::Error`.
