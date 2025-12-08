## ADDED Requirements

### Requirement: Full Error Context
When an error occurs, the user MUST be presented with the full chain of causes, not just the top-level error message. This ensures that context wrappers (e.g. "Failed to resolve chart dependencies") do not hide specific details (e.g. "Chart A depends on unknown chart B").

#### Scenario: Dependency Chain Failure
Given a chart "frontend" which depends on "missing-backend"
And "missing-backend" is not defined in `vesshelm.yaml`
When I run `vesshelm deploy`
Then the output should contain "Failed to resolve chart dependencies"
And the output should contain "frontend"
And the output should contain "missing-backend"
And the output should contain "depends on unknown chart"
