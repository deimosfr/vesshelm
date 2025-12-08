## ADDED Requirements

### Requirement: Validate Configuration
The tool MUST provide a `validate` command that checks the syntax and semantic validity of the `vesshelm.yaml` configuration file.

#### Scenario: Validate valid config
- **GIVEN** a valid `vesshelm.yaml`
- **WHEN** user runs `vesshelm validate`
- **THEN** the command exits with success code
- **AND** prints "Configuration is valid"

#### Scenario: Validate invalid config
- **GIVEN** a `vesshelm.yaml` with missing required fields or invalid values
- **WHEN** user runs `vesshelm validate`
- **THEN** the command exits with failure code
- **AND** prints specific validation errors
