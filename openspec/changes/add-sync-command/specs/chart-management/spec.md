## ADDED Requirements

### Requirement: Sync Charts
The `sync` command MUST download and synchronize all charts defined in the configuration file to their specified destinations.

#### Scenario: Basic sync
- **GIVEN** a `vesshelm.yaml` with valid chart definitions
- **WHEN** user runs `vesshelm sync`
- **THEN** all charts are downloaded to the correct paths
- **AND** the command reports success

### Requirement: Safe Chart Replacement
When syncing a chart that already exists locally, the tool MUST use a safe replacement strategy: download and verify the new chart first, then delete the old chart folder, and finally move the new chart into place.

#### Scenario: Update existing chart
- **GIVEN** a chart `nginx` exists at `charts/nginx`
- **WHEN** user runs `vesshelm sync` and `nginx` version has changed
- **THEN** new version is downloaded to a temporary location
- **AND** `charts/nginx` is deleted
- **AND** New version is moved to `charts/nginx`
- **AND** The operation ensures no partial files are left if download fails

### Requirement: Repository Support
The tool MUST support downloading charts from Helm repositories, Git repositories, and OCI registries as defined in the `repositories` section of the config.

#### Scenario: Support various sources
- **GIVEN** config defines Helm, Git, and OCI repositories
- **WHEN** `vesshelm sync` is run
- **THEN** charts from all source types are correctly fetched
