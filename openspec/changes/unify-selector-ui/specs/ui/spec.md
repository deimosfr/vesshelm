# UI Consistency

## MODIFIED Requirements

### Requirement: Chart Selection Display

The application MUST display chart items consistently across all interactive commands (`deploy`, `delete`, `uninstall`).

#### Scenario: Interactive Selection
When the user runs `vesshelm delete` (or deploy/uninstall) without arguments:
- The list of charts MUST be displayed.
- Each item MUST follow the format: `[Name] ([Namespace])`.
- The list MUST NOT contain broken ANSI escape codes (e.g. `[2m`).
- The `[Name]` component SHOULD be bold.
- The `([Namespace])` component SHOULD be dimmed/grey.
- The selector MUST visually distinguish selected items without breaking ANSI codes.
