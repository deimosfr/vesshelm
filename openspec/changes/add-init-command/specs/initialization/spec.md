## ADDED Requirements

### Requirement: Check Helm Availability
The `init` command MUST check if the `helm` executable is available in the system's `PATH`. If it is not found, the command MUST exit with an error.

#### Scenario: Helm is missing
- **GIVEN** `helm` is not installed or not in `PATH`
- **WHEN** user runs `vesshelm init`
- **THEN** the command exits with a non-zero status code
- **AND** an error message indicating `helm` is missing is displayed

#### Scenario: Helm is present
- **GIVEN** `helm` is installed and in `PATH`
- **WHEN** user runs `vesshelm init`
- **THEN** the command proceeds without erroring on dependency check

### Requirement: Create Default Configuration
The `init` command MUST create a default configuration file (e.g., `vesshelm.yaml`) if one does not already exist in the current directory or expected location.

#### Scenario: Configuration file missing
- **GIVEN** no `vesshelm.yaml` file exists in the current directory
- **WHEN** user runs `vesshelm init`
- **THEN** a new `vesshelm.yaml` file is created with default values
- **AND** the command reports success

#### Scenario: Configuration file exists
- **GIVEN** a `vesshelm.yaml` file already exists
- **WHEN** user runs `vesshelm init`
- **THEN** the existing file is NOT overwritten
- **AND** the command reports that configuration already exists (or silently succeeds if idempotent)
