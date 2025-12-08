## MODIFIED Requirements
### Requirement: CLI Commands
The CLI logic MUST be separated into distinct modules for each command (e.g., `init`, `sync`).

#### Scenario: Code structure check
- **GIVEN** the source code
- **WHEN** inspected
- **THEN** `src/cli/commands/init.rs` exists
- **AND** `src/cli/commands/sync.rs` exists
