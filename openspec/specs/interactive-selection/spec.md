# interactive-selection Specification

## Purpose
TBD - created by archiving change interactive-deploy-selection. Update Purpose after archive.
## Requirements
### Requirement: Deploy Command Default Behavior
The deploy command MUST default to interactive mode when no arguments are provided.

#### Scenario: No arguments provided
Given the user runs `vesshelm deploy`
And no arguments are provided
Then the system SHALL display an interactive fuzzy selector
And the selector SHALL list "All charts" and all available individual charts
And the system SHALL NOT automatically deploy all charts without user selection
And the selector list SHALL be sorted alphabetically by chart name
And the selector items SHALL be formatted as "Name (Namespace)"

#### Scenario: Selector Visual Style
Given the interactive selector is displayed
Then the charts SHALL be listed in alphabetical order (e.g. "api", "db", "frontend")
And each entry SHALL show the namespace in parentheses
And "All charts" SHALL be the first option regardless of sorting

#### Scenario: Selecting "All charts"
Given the interactive selector is displayed
When the user selects "All charts"
Then the system SHALL proceed to deploy all charts in the configuration

#### Scenario: Selecting a specific chart
Given the interactive selector is displayed
When the user selects a specific chart (e.g., "nginx")
Then the system SHALL deploy only the "nginx" chart
And the system SHALL filter the deployment list to only this chart

#### Scenario: User cancellation
Given the interactive selector is displayed
When the user cancels the selection (e.g., Esc or Ctrl+C)
Then the system SHALL abort the operation without deploying anything
