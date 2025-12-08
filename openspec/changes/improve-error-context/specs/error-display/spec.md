## MODIFIED Requirements

### Requirement: Explicit Error Locations
All configuration and runtime errors MUST provide explicit context to help the user identify the source of the problem, including:
- File paths for IO and parsing errors
- Chart names/IDs for logic or validation errors relating to specific charts
- Line numbers/keys where applicable and available

#### Scenario: Parse error includes file path
Given a malformed `vesshelm.yaml` file
When I run `vesshelm deploy`
Then the error message MUST contain the path to `vesshelm.yaml`

#### Scenario: Default destination missing includes chart
Given a chart "my-chart" with no destination override
And `vesshelm.yaml` has no "default" destination
When I run `vesshelm deploy`
Then the error message MUST contain "my-chart" and "default" destination not found

#### Scenario: Version update boundary error includes chart
Given a `vesshelm.yaml` where "my-chart" version cannot be safely updated due to boundary ambiguity
When I run `vesshelm check-updates` (or internal update logic)
Then the error message MUST contain "my-chart"
