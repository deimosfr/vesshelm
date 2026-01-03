# Pause Deploy on Error

## Problem
Currently, when a deployment fails, Vesshelm exits immediately. This makes it difficult to debug the state of the cluster or the helm release at the moment of failure, especially in complex deployments.

## Solution
Introduce a pause mechanism when a deployment error occurs. This pause will allow the user to inspect the system before Vesshelm exits.
This behavior will be enabled by default but can be disabled via configuration or by running in non-interactive mode.

## Risks
- Scripts or CI pipelines that do not use `--no-interactive` might hang on error if they run in a TTY-like environment. Users must ensure they use the appropriate flags for automation.
