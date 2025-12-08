# Command Summary Factorization

## ADDED Requirements

### Requirement: Shared Utility
The command summary MUST be managed by a shared utility to ensure consistent formatting across all CLI commands.

#### Scenario: Printing Summary
Given a command has completed execution
And it has collected statistics (e.g., synced, failed, skipped)
When the summary is printed
Then the output must follow a standard format:
  - Header "Summary:" (Bold/Bright)
  - Key-Value pairs aligned vertically
  - Values colored based on context (Green for success/synced, Red for failure, Yellow for skipped/warn)

### Requirement: Alignment
The summary output MUST align values vertically by padding keys.

#### Scenario: Alignment Display
Given multiple statistics keys of different lengths (e.g., "Outdated", "Up to date")
When the summary is displayed
Then the values must be aligned vertically by padding the keys
Example:
  Summary:
    Outdated:   1
    Up to date: 0
    Skipped:    0
