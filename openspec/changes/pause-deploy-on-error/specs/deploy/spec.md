# Deploy Debug Pause

## ADDED Requirements

### Requirement: Pause on deployment error
The deploy command MUST pause execution and wait for user input when a deployment fails, to allow for debugging.

#### Scenario: Default behavior (Interactive)
Given a vesshelm configuration with `deploy_debug_pause` set to `true` (default)
And I am running `vesshelm deploy` in an interactive terminal
When a chart deployment fails
Then Vesshelm should print the error
And Vesshelm should print a message indicating it is paused for debugging
And Vesshelm should wait for user input (e.g., Press Enter)
And after input is received, Vesshelm should exit with the error.

#### Scenario: No interactive mode
Given I am running `vesshelm deploy --no-interactive`
When a chart deployment fails
Then Vesshelm should NOT pause
And Vesshelm should exit with the error immediately.

#### Scenario: Configured to not pause
Given a vesshelm configuration with `deploy_debug_pause` set to `false`
And I am running `vesshelm deploy`
When a chart deployment fails
Then Vesshelm should NOT pause
And Vesshelm should exit with the error immediately.
