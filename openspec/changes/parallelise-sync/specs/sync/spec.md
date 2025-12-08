## ADDED Requirements
### Requirement: Parallel Sync
The tool SHALL support synchronizing multiple charts in parallel to improve performance.

#### Scenario: User Defines Jobs
- **WHEN** user runs `vesshelm sync --jobs <N>`
- **THEN** up to N charts are synchronized concurrently.

#### Scenario: Default Jobs
- **WHEN** user runs `vesshelm sync` without specifying jobs
- **THEN** charts are synchronized concurrently with a default concurrency limit (e.g. 5).
