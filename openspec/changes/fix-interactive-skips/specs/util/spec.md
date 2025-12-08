# Spec Delta: Interactive Skips

## MODIFIED Requirements

### Interactive Prompts
Interactive prompts MUST NOT interfere with the progress bar display.

#### Scenario: Prompting during Progress
When the application prompts the user (e.g., "Do you want to deploy?"):
- The progress bar MUST be temporarily suspended (cleared/hidden).
- The prompt MUST appear cleanly on standard output.
- The progress bar MUST reappear after the user responds.
