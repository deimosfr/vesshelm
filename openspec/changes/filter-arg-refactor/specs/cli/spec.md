# CLI Filtering

## MODIFIED Requirements

### Requirement: Filter charts by name
The `check-updates`, `sync`, and `deploy` commands MUST accept optional positional arguments to filter the operation to specific charts.
- If no arguments are provided, the operation applies to ALL charts.
- If arguments are provided, the operation applies ONLY to the specified charts.
- The `--only` flag is REMOVED.

#### Scenario: Filter by positional arguments
Given a list of charts `foo`, `bar`, `baz`
When the user runs `vesshelm sync foo baz`
Then only `foo` and `baz` are synced.

#### Scenario: No filter arguments
Given a list of charts `foo`, `bar`
When the user runs `vesshelm sync`
Then both `foo` and `bar` are synced.

#### Scenario: Legacy flag usage
When the user runs `vesshelm sync --only foo`
Then the command fails with an unknown argument error.
