## MODIFIED Requirements

### Requirement: Repository Support
The tool MUST support downloading charts from Helm repositories, Git repositories, and OCI registries. The tool MUST use native libraries for Git operations and file system manipulations, avoiding external binary dependencies except for the `helm` executable.

#### Scenario: Git Clone with Lib
- **GIVEN** a config with a Git repository
- **WHEN** user runs `vesshelm sync`
- **THEN** the repository is cloned using the embedded Git library
- **AND** no `git` executable is spawned
