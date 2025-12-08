# Deploy Capability

## MODIFIED Requirements

### Requirement: Progress Bar Count
The progress bar must reflect the actual number of charts to be processed after filtering.

#### Scenario: Progress bar uses filtered count
Given the user provides `--only <chart>`
When the deployment starts
Then the progress bar total should equal the number of charts matching the filter
And the progress bar should not count charts excluded by the filter
