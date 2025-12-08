# Error Handling Improvements

## ADDED Requirements

#### Requirement: Fancy Error Output
All CLI errors must be formatted to be visually distinct and readable.
- **Scenario:** When a command fails, the output should start with **"Error:"** in bold red.
- **Scenario:** The error message body should follow the header.

#### Requirement: Readable Validation Errors
Configuration validation errors must be presented as a clear list of issues rather than a raw dump.
- **Scenario:** When `vesshelm.yaml` is invalid, the specific validation errors should be listed bullet-by-bullet.
- **Scenario:** Field names and error messages should be formatted for readability (e.g. `chart_repo_not_found` -> "Chart repository not found").
- **Scenario:** `__all__` errors should be handled gracefully.

