## ADDED Requirements

### Requirement: Helm Diff Plugin Installation

The `init` command MUST ensure the `helm-diff` plugin is installed to support the default configuration.

#### Scenario: Plugin missing
Given the `helm-diff` plugin is not installed
When I run `vesshelm init`
Then the command should install the `helm-diff` plugin
And the installation should use `--verify=false`
And report successful installation

#### Scenario: Plugin already installed
Given the `helm-diff` plugin is already installed
When I run `vesshelm init`
Then the command should not attempt to install the plugin again
And report that the plugin is already installed or checked
