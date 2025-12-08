# Uninstall Chart Capability

## MODIFIED Requirements

### Requirement: Check dependencies before uninstall
The system MUST check if other charts depend on the chart being uninstalled.

#### Scenario: Uninstall with dependencies
Given a chart `A`
And a chart `B` that depends on `A`
When I run `vesshelm uninstall A`
Then I should see a warning that `B` depends on `A`
And I should be asked for confirmation

#### Scenario: Uninstall without dependencies
Given a chart `A` with no dependents
When I run `vesshelm uninstall A`
Then I should see a message confirming no other charts depend on `A`
And I should be asked for confirmation
