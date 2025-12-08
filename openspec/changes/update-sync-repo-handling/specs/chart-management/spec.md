## MODIFIED Requirements

### Requirement: Repository Support
The tool MUST support downloading charts from Helm repositories, Git repositories, and OCI registries. The tool MUST NOT error if a repository definition already exists; instead, it MUST inform the user that the repository already exists.

#### Scenario: Existing repository
- **GIVEN** a Helm repository `stable` is already added to local Helm
- **WHEN** user runs `vesshelm sync` containing `stable` repo definition
- **THEN** the command proceeds without error
- **AND** a message "Repository 'stable' already exists" is displayed (or logged)
- **AND** chart pulling proceeds normally
