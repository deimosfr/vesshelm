## ADDED Requirements

### Requirement: Progress Bar UX
The system MUST provide a persistent progress bar that indicates the status of long-running operations. The progress bar MUST NOT interfere with interactive prompts and MUST be capable of being disabled.

#### Scenario: User Disables Progress Bar
- **Given** I run `vesshelm sync` or `vesshelm deploy`
- **And** I pass the `--no-progress` flag
- **Then** the persistent progress bar should NOT be displayed.
- **And** log messages should still be printed to stdout/stderr.

#### Scenario: Visual Separation
- **When** the `sync` or `deploy` command completes successfully
- **Then** a blank line should be printed BEFORE the final "Alignment completed" or summary message.

#### Scenario: Interactive Deployment Prompt
- **Given** I run `vesshelm deploy` (without `--no-progress` and without `--no-interactive` or `--dry-run`)
- **And** the progress bar is displayed
- **When** the CLI needs to ask for confirmation ("Do you want to deploy...?")
- **Then** a blank line should be printed before the prompt question.
- **And** the progress bar should be hidden/cleared while waiting for user input.
- **So that** the prompt is clearly visible and the user input is not overwritten by progress bar updates.
- **And** after the user responds, the progress bar should resume (if not finished).
