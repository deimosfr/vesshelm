# Spec: DAG Ordering

## ADDED Requirements

### Requirement: Deploy command must respect DAG order
The deploy command MUST execute deployments in an order that satisfies all DAG dependencies defined in the charts.

#### Scenario: Deploy respects dependency order
Given a configuration with charts A and B
And B depends on A
When I run `vesshelm deploy`
Then A should be deployed before B

#### Scenario: Deploy specific chart with dependencies
Given a configuration with charts A and B
And B depends on A
And A is not deployed
When I run `vesshelm deploy --charts B`
Then I should be warned that A is not selected? (Or A should be automatically selected? - Clarification needed)


## MODIFIED Requirements
None
