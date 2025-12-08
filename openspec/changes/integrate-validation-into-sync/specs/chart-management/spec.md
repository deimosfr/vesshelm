## MODIFIED Requirements

### Requirement: Sync Safety
The `sync` command MUST validate the configuration file before performing any changes to the file system. If the configuration is invalid, the command MUST abort and report errors.

#### Scenario: Sync with invalid config
- **GIVEN** a `vesshelm.yaml` with invalid references
- **WHEN** user runs `vesshelm sync`
- **THEN** the command runs validation
- **AND** reports validation errors
- **AND** does NOT attempt to sync any charts
