# Install Helm Diff Plugin during Init

## Goal
Automatically verify and install the `helm-diff` plugin during the `vesshelm init` command execution to ensure the default configuration (which enables diffing) works out of the box.

## Motivation
Vesshelm's default configuration enables `diff_enabled: true`. Users running `vesshelm init` followed by `vesshelm deploy` might encounter errors if they don't have the `helm-diff` plugin installed. Automating this step improves the onboarding experience and reliability.

## External Behaviors
- The `init` command will check for the presence of the `diff` plugin in the Helm plugin list.
- If missing, it will attempt to install the plugin from `https://github.com/databus23/helm-diff` using `--verify=false` to bypass checksum verification (as the plugin release might not have it or we want to avoid issues).
- It will report success or failure of this operation.
