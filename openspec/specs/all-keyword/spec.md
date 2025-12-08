# all-keyword Specification

## Purpose
TBD - created by archiving change interactive-deploy-selection. Update Purpose after archive.
## Requirements
### Requirement: Deploy Command "All" Keyword
A new reserved keyword "all" is added to the `deploy` command arguments which MUST explicitly trigger the deployment of all charts.

#### Scenario: deploy all
Given the user runs `vesshelm deploy all`
Then the system SHALL deploy all charts defined in the configuration
And the system SHALL skip the interactive selector

#### Scenario: deploy all with other charts
Given the user runs `vesshelm deploy all nginx`
Then the system SHALL deploy all charts
And the "all" keyword SHALL take precedence over individual chart names (effectively ignoring specific charts since they are included in "all")

#### Scenario: deploy "all" in non-interactive mode
Given the user runs `vesshelm deploy all --no-interactive`
Then the system SHALL deploy all charts without pausing for confirmation (standard behavior)
