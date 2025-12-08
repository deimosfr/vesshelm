# Update Helm Repos on Sync

## Summary
Update the `sync` command to run `helm repo update` before pulling charts, ensuring the local repository cache is up-to-date with the remote. This should only run if a chart actually needs to be synced.

## Motivation
Currently, `vesshelm sync` might fail to pull the latest version of a chart if the local helm repository cache is outdated, even if the user asks for a specific version that exists remotely. Users have to manually run `helm repo update`.

## Proposed Solution
Modify the sync logic to trigger `helm repo update` once per execution, but only if at least one chart is not skipped (i.e., not up-to-date or ignored).
