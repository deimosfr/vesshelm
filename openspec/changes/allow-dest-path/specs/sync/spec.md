# Sync Capability
## MODIFIED Requirements

### Requirement: Chart Destination Resolution
The system MUST resolve the destination directory for a chart by checking the `dest` field.
#### Scenario: Named Destination
GIVEN a chart with `dest: "plugins"`
AND a destination named "plugins" pointing to "libs/plugins"
WHEN `vessel sync` is run
THEN the chart is synced to "libs/plugins/<chart-name>"

#### Scenario: Direct Path Destination
GIVEN a chart with `dest: "libs/custom-plugins"`
AND no destination named "libs/custom-plugins" exists
WHEN `vessel sync` is run
THEN the chart is synced to "libs/custom-plugins/<chart-name>"
