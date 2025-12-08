# deploy Specification

## Purpose
TBD - created by archiving change pause-deploy-on-error. Update Purpose after archive.
## Requirements
### Requirement: Pause on deployment error
The deploy command MUST pause execution and wait for user input when a deployment fails, to allow for debugging. Temporary values files MUST be preserved for inspection during the pause.

#### Scenario: Debugging a failed deployment
Given a chart deployment that fails
And `deploy_debug_pause` is set to `true`
When the deployment fails
Then the process should pause
And the temporary values files should be preserved on disk
And the console output should display the path to the temporary values files
And the files should be deleted only after the user resumes the process
