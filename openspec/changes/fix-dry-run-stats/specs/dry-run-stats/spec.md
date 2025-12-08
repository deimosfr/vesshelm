## MODIFIED Requirements

### Requirement: Deploy Command Summary

The deploy command summary MUST accurately reflect the actions taken, especially during dry-run.

#### Scenario: Dry Run Count
Given I run `vesshelm deploy --dry-run`
When the command completes
Then the "Deployed" count in the summary MUST be 0
And the charts that were processed in dry-run MUST NOT increment the Deployed count
