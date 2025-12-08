## MODIFIED Requirements

### Requirement: Path Resolution
The `delete` command MUST determine the correct filesystem path for a chart based on its configuration.

#### Scenario: Local Chart
Given a chart is configured as a local chart (no `repo_name`, `chart_path` defined)
Then `delete` MUST target the path specified in `chart_path`.

#### Scenario: Explicit Destination Adaptation
Given a chart has an explicit `dest` configuration (not a named alias)
When resolving the path
Then `delete` MUST check if the directory `dest` exists and `dest/name` does not.
And allow targeting `dest` directly if it appears to be the intended chart directory.
Otherwise, it defaults to `dest/name`.

### Requirement: Directory Status
The summary MUST indicate the state of the target directory.

#### Scenario: Missing Directory
Given the resolved path does not exist
The summary MUST display "Status: Missing (already deleted)" or equivalent.

#### Scenario: Present Directory
Given the resolved path exists
The summary MUST display "Status: Present".
