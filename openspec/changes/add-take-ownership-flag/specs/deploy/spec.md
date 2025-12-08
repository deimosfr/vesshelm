# Spec Delta: Take Ownership Flag

## MODIFIED Requirements

### Deployment Arguments
The `deploy` command MUST accept optional flags to modify the underlying Helm execution.

#### Scenario: Take Ownership
When `vesshelm deploy --take-ownership` is run:
- The flag `--take-ownership` MUST be appended to the generated Helm command arguments for every chart.
- This allows passing ownership resolution flags (supported by specific Helm versions or plugins) without config modification.
