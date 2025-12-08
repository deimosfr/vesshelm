## ADDED Requirements

### Requirement: Configuration Schema
The application MUST support a configuration file (YAML) that defines `repositories`, `charts`, and `destinations`.

#### Scenario: Valid Configuration
- **GIVEN** a valid `vesshelm.yaml` file
- **WHEN** the application loads configuration
- **THEN** it correctly parses the list of repositories, charts, and destinations
- **AND** Default destination is recognized

### Requirement: Chart Definitions
Chart definitions in the configuration MUST include `name`, `repo_name`, `version`, and `namespace`. Optional fields include `dest`, `chart_path` (for git), and `no_sync`.

#### Scenario: Required fields
- **GIVEN** a chart entry in configuration
- **WHEN** validating the config
- **THEN** `name`, `repo_name`, `version`, and `namespace` are required
- **AND** Missing any raises a validation error

### Requirement: Destinations
The configuration MUST allow defining named destinations, which resolve to local filesystem paths.

#### Scenario: Resolve destination
- **GIVEN** a destination named "custom" mapped to "./custom-charts"
- **AND** a chart specifies `dest: custom`
- **WHEN** syncing
- **THEN** the chart is downloaded to "./custom-charts/<chart-name>"
