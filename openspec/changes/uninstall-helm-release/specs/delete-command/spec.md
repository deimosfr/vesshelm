## MODIFIED Requirements

### Requirement: Release Uninstallation
The `delete` command MUST offer to uninstall the associated Helm release.

#### Scenario: User Prompt
Given the user initiates a delete command
The command MUST prompt "Do you also want to uninstall the Helm release?" (Default: No).

#### Scenario: Execution Order
Given the user confirms release uninstallation
The command MUST attempt to uninstall the release via Helm.
And only proceed to remove the chart from `vesshelm.yaml` and filesystem if:
1.  The uninstallation succeeds.
2.  OR the release was already not found.

#### Scenario: Failure Handling
Given the release uninstallation fails (and is not a "not found" error)
The command MUST abort and NOT remove the chart from `vesshelm.yaml` or filesystem.
And display an error message explaining why the config was preserved.

#### Scenario: Summary Display
Given the user confirms release uninstallation
The deletion summary MUST explicitly state "Uninstall release: Yes".
And the Action description MUST start with "Uninstall Helm release".
