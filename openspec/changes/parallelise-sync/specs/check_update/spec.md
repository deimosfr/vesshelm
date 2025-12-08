## ADDED Requirements
### Requirement: Parallel Check Updates
The tool SHALL support checking for updates for multiple charts in parallel to improve performance.

#### Scenario: User Defines Jobs
- **WHEN** user runs `vesshelm check-update --jobs <N>`
- **THEN** up to N charts are checked for updates concurrently.

#### Scenario: Default Jobs
- **WHEN** user runs `vesshelm check-update` without specifying jobs
- **THEN** charts are checked concurrently with a default concurrency limit (e.g. 5).
