## MODIFIED Requirements

### [CLI] Deploy Command

#### Scenario: User disables diff via flag
GIVEN `vesshelm.yaml` has `diff_enabled: true`
WHEN I run `vesshelm deploy --no-diff`
THEN the deployment proceeds without showing the diff
THEN the deployment proceeds without showing the diff
AND the interactive confirmation is skipped (Constraint: exiting logic couples confirmation to diff view)

#### Scenario: User runs dry-run with no-diff
GIVEN `vesshelm.yaml` has `diff_enabled: true`
WHEN I run `vesshelm deploy --dry-run --no-diff`
THEN the command fails with an argument conflict error
    - *Constraint*: `dry_run` and `no_diff` are mutually exclusive.
