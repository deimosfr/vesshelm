# Fix Check Updates Repository Registration

## Context

When running `vesshelm check-updates`, the system attempts to search for chart versions in Helm repositories. However, it currently assumes that all repositories defined in `vesshelm.yaml` are already added to the local Helm environment. If a user adds a new repository to the config and immediately runs `check-updates`, it fails with "Chart not found" because the repository hasn't been added/registered with Helm yet.

## Problem

`vesshelm check-updates` only runs `helm repo update`, which updates _existing_ repositories but does not add new ones. This leads to confusing errors when checking updates for charts from newly added repositories (like `mealie` from `deimosfr-charts` in the user report).

## Solution

Update `check-updates` to iterate through all defined repositories `vesshelm.yaml` and explicitly run `helm repo add` for each one (or specifically for those of type `Helm`) before running `helm repo update`. This ensures the environment is correctly prepared.
