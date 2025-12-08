# Fix Check Updates Pre-Release Lookup

## Context

Some charts, like `mealie` in `deimosfr-charts`, only publish versions that appear as pre-releases to Helm (e.g., `3.9.2-1`). Helm's default search behavior hides these versions unless the `--devel` flag is used.

## Problem

`vesshelm check-updates` currently runs `helm search repo` without `--devel`. If a repository contains _only_ pre-release versions (or if the user wants to update to/from a pre-release), `vesshelm` fails to find the chart entirely, reporting "Chart not found".

## Solution

Update the `check-updates` logic to attempt a lookup with `--devel` if the standard lookup returns no results. This ensures that users tracking pre-release charts (or charts that use pre-release versioning semantics) can still check for updates.
