## ADDED Requirements

### Requirement: Standardized Step Output
The CLI MUST use a centralized function to format "step" messages (titles), ensuring a consistent visual style (e.g. `==> Message`) across all commands.

#### Scenario: Visual Consistency
- **WHEN** the user runs a command like `deploy`, `delete`, or `uninstall`
- **THEN** all step titles are prefixed with a colored `==>` arrow and are bolded consistently
- **AND** no hardcoded formatting is used in individual commands
