## ADDED Requirements

### Requirement: Alphabetical Execution
The `check-updates` command MUST process and display charts in alphabetical order by their name, regardless of their order in the configuration file.

#### Scenario: Unsorted Configuration
Given a configuration with charts ["zulu", "alpha", "bravo"]
When I run `vesshelm check-updates`
Then the output should show "checking alpha..."
And then the output should show "checking bravo..."
And then the output should show "checking zulu..."
