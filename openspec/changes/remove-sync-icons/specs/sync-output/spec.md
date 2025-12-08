## MODIFIED Requirements

### Requirement: Sync Command Output

The `sync` command output MUST be simplified and aligned.

#### Scenario: Successful sync
Given I run `vesshelm sync`
When a chart syncs successfully
Then the output line MUST start with `[OK]  ` (with two spaces padding)
And the chart name MUST follow immediately

#### Scenario: Failed sync
Given I run `vesshelm sync`
When a chart fails to sync
Then the output line MUST start with `[FAIL] ` (with one space padding)
And the chart name MUST follow immediately

#### Scenario: Skipped chart
Given I run `vesshelm sync`
When a chart is skipped
Then the output line MUST start with `[SKIP] ` (with one space padding)
And the chart name MUST follow immediately
