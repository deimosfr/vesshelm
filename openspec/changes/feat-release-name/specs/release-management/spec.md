## ADDED Requirements

### Requirement: Custom Release Name
The system MUST allow users to specify a custom Helm release name for a chart, separate from its logical name or directory.

#### Scenario: Default Release Name
Given a chart config with `name: "my-chart"` and no `release_name`
When the chart is deployed
Then the Helm release name MUST be "my-chart"

#### Scenario: Custom Release Name
Given a chart config with `name: "my-chart"` and `release_name: "custom-release"`
When the chart is deployed
Then the Helm release name MUST be "custom-release"
And the `{{ name }}` variable in Helm args MUST resolve to "custom-release"
And the `{{ chart_name }}` variable MUST resolve to "my-chart"

#### Scenario: Uninstall Custom Release
Given a chart deployed with `release_name: "custom-release"`
When the chart is uninstalled via `vesshelm uninstall my-chart`
Then the system MUST uninstall the Helm release "custom-release"
