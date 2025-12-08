## MODIFIED Requirements

### Requirement: CLI Commands
The CLI MUST support `init` and `sync` commands.

#### Scenario: Sync command available
- **WHEN** user runs `vesshelm --help`
- **THEN** `sync` is listed as an available command
- **AND** `init` is listed as an available command
