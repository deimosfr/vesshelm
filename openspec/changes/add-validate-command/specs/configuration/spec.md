## MODIFIED Requirements

### Requirement: Configuration Validation
The configuration MUST be validated for:
- **Repositories**: Unique names, valid URLs.
- **Destinations**: Unique names, valid paths.
- **Charts**:
    - `name` + `namespace`: combination MUST be unique across the list.
    - `repo_name`: must exist in repositories list.
    - `version`: non-empty.
    - `dest`: must exist in destinations list.

#### Scenario: Invalid repo URL
- **GIVEN** a repo with invalid URL
- **WHEN** validation is run
- **THEN** an error is reported

#### Scenario: Missing repo reference
- **GIVEN** a chart referencing a non-existent repo
- **WHEN** validation is run
- **THEN** an error is reported indicating the missing repository name

### Requirement: Error Reporting
Validation errors MUST be descriptive and user-friendly.
- For duplicate charts, the error MUST explicitly state the duplicated name and namespace.
