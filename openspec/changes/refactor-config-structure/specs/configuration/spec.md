# Configuration Structure Update

## MODIFIED Requirements

### Configuration Schema
#### Scenario: Define Vesshelm global settings
- Given a `vesshelm.yaml` file
- When defining global execution settings
- Then the top-level key MUST be `vesshelm` instead of `helm`
- And the arguments key MUST be `helm_args` instead of `args`
- And `diff_enabled` and `diff_args` MUST be retained under `vesshelm`
