# Deploy Capability

## MODIFIED Requirements

### Deployment Summary
The deployment summary must accurately reflect the outcome of each chart processing.

#### Scenario: Summary distinguishes between skipped and ignored charts
Given a deployment run with interactive mode enabled
When the user answers "No" to a deployment prompt
Then the chart is counted as "Ignored" in the summary
And the chart is NOT counted as "Skipped" in the summary
And the summary displays the count of "Ignored" charts separately

#### Scenario: Configuration skip is counted as Skipped
Given a chart with `no_deploy: true`
When the deployment runs
Then the chart is counted as "Skipped" in the summary
And the chart is NOT counted as "Ignored"

#### Scenario: No changes skip is counted as Skipped
Given a chart with `diff_enabled: true`
And the chart has no changes compared to the cluster
When the deployment runs
Then the chart is counted as "Skipped" in the summary
And the chart is NOT counted as "Ignored"
