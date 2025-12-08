## ADDED Requirements
### Requirement: Check Update Summary
The tool SHALL print a summary of the check update results at the end of execution.

#### Scenario: Summary Output
- **WHEN** user runs `vesshelm check-update`
- **THEN** a summary is printed showing the count of charts that are:
    - Outdated
    - Up to date
    - Skipped
    - Failed

#### Scenario: Alignment
- **WHEN** the summary is printed
- **THEN** the counts are aligned for readability (e.g. colons aligned).
