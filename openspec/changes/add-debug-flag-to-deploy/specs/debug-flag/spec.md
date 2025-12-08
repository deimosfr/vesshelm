# Spec: Debug Flag for Deploy Command

## ADDED Requirements

### Requirement: detailed-debug-logging
The `deploy` command MUST support a `--debug` flag that enables verbose logging in the underlying Helm operation.

#### Scenario: User runs deploy with debug flag
Given a valid chart configuration
When I run `vesshelm deploy --debug`
Then the underlying `helm upgrade` command SHALL include the `--debug` argument
And I SHALL see verbose output from Helm
