# Spec Delta: Fix Take Ownership

## ADDED Requirements

### Requirement: Deployment Execution
The deployment process MUST correctly utilize all provided CLI flags.

#### Scenario: Take Ownership Propagation
When `vessel deploy --take-ownership` is invoked:
- The execution logic MUST append the `--take-ownership` flag to the underlying Helm command command (e.g. `helm upgrade ... --take-ownership`).
- This MUST occur regardless of whether `dry-run` is active or not (though dry-run only prints it).
