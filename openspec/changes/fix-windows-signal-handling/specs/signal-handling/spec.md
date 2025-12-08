# Signal Handling

## ADDED Requirements

### Requirement: Cross-Platform Interruption
The CLI MUST handle interruption signals supportively on both Unix and Windows systems.

#### Scenario: Unix Interruption
Given the user is running `vesshelm deploy` on Linux or macOS
When they press Ctrl+C
Then the command should catch SIGINT
And gracefully terminate the child Helm process.

#### Scenario: Windows Interruption
Given the user is running `vesshelm deploy` on Windows
When they press Ctrl+C
Then the command should catch the interruption
And gracefully terminate the child Helm process.
