# CLI Commands

## ADDED Requirements

### Requirement: Uninstall Command
The tool MUST allow uninstalling a chart release that is managed by the configuration.

#### Scenario: Uninstall existing chart
Given a chart named "my-chart" is defined in `vesshelm.yaml`
When the user runs `vesshelm uninstall my-chart`
Then the tool asks for confirmation "Are you sure you want to uninstall my-chart? [y/N]"
And if users answers "y"
Then the tool runs `helm uninstall my-chart` with the namespace defined in the configuration

#### Scenario: Uninstall unknown chart
Given "unknown-chart" is not defined in `vesshelm.yaml`
When the user runs `vesshelm uninstall unknown-chart`
Then the tool exits with an error indicating the chart is not found

#### Scenario: Abort uninstall
Given the user runs `vesshelm uninstall my-chart`
When the prompt appears and user answers "N"
Then the tool aborts and does not run `helm uninstall`
