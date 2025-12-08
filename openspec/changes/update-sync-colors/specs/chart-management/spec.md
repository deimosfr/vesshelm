# Chart Management

## MODIFIED Requirements

### Sync Charts
The `sync` command downloads charts to the local filesystem.

#### Scenario: Displays fancy colors
Given I run `vesshelm sync`
Then the output should use specific colors for statuses
And `(up to date)` should be green
And `(local chart)` should be blue
And `[SKIP]` should be grey
