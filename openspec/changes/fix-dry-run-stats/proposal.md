# Fix Dry-Run Stats Configuration

## Goal
Ensure that the `deployed` count in the summary remains 0 when running `vesshelm deploy --dry-run`, to avoid confusion.

## Motivation
Currently, when running a dry-run deployment, the summary reports charts as "Deployed". This is confusing because no actual changes are applied to the cluster. The summary should reflect that 0 charts were actually deployed.

## External Behaviors
- When `vesshelm deploy --dry-run` is executed:
    - The "Deployed" count in the summary MUST be 0.
    - Charts that would have been deployed should be counted separately or just not affect the "Deployed" metric. (Given the requirement is just "deployed stays to 0", we will ensure it doesn't increment).
    - We will treat dry-run actions as "Ignored" or simply not count them towards "Deployed" or "Skipped" in the confusing way.
    - Actually, existing "Ignored" category seems appropriate or we can just leave them out of the counts if they don't fit "Failed" or "Skipped".
    - However, user simply stated "deployed is above 0, while it should stay to 0".
    - I will implement a change where dry-run returns a status that does not increment `deployed_count`.
