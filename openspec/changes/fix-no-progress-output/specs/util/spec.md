# Spec Delta: Progress Output

## MODIFIED Requirements

### Progress Tracking
The application MUST provide a way to disable the visual progress bar without suppressing other output.

#### Scenario: No Progress Mode
When `vesshelm deploy --no-progress` is executed:
- The visual progress bar/spinner MUST NOT be displayed.
- Standard log messages, diffs, and prompts logged via the tracker's `println` method MUST still be printed to stdout.
