# Deploy Command Specification

## ADDED Requirements

#### Requirement: Deployment Summary
The `deploy` command MUST display a summary of the operation counts (Deployed, Failed, Skipped) at the end of execution.

#### Scenario: Successful Deployment
Given a `vesshelm.yaml` with 2 charts
When `vesshelm deploy` is run
Then the output ends with:
  Summary:
    Deployed: 2
    Failed:   0
    Skipped:  0

#### Scenario: Skipped Deployment
Given a chart with `no_deploy: true`
When `vesshelm deploy` is run
Then the summary shows:
  Skipped: 1

#### Scenario: Failed Deployment
Given a chart that fails to deploy
When `vesshelm deploy` is run
Then the summary shows:
  Failed: 1
