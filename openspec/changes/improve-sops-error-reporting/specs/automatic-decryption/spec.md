# Automatic Decryption

## ADDED Requirements

### Requirement: Detailed Error Reporting for Rendering Failures
When rendering a values file (encrypted or plain) fails, the system MUST report the specific cause of the failure, including missing variables or template syntax errors.

#### Scenario: Verify failure detail for missing variables
Given a values file (encrypted or plain) that references a missing variable `{{ unknown_var }}`
When `vesshelm` attempts to render this file
Then the deployment should fail
And the error output should explicitly mention "unknown_var" or "undefined value"
