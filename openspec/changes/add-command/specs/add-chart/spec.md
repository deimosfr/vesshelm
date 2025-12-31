# Spec: Add Chart

**capability-id**: `add-chart`

## ADDED Requirements

### Requirement: Interactive Wizard
The `add` command MUST provide an interactive wizard to collect chart information.

#### Scenario: User starts wizard
Given the user runs `vesshelm add`
Then the system prompts for an Artifact Hub URL

### Requirement: Source Selection
The `add` command MUST allow the user to choose the source type.

#### Scenario: Select Source
Given the user runs `vesshelm add`
Then the system prompts "Select source type"
And the options are "Artifact Hub (default)", "Git", "OCI"

### Requirement: Artifact Hub Parsing
The `add` command MUST parse the provided Artifact Hub URL to extract chart details (if Source is Artifact Hub).

#### Scenario: Valid URL provided
[As before...]

### Requirement: Git Configuration
The `add` command MUST support adding charts from Git repositories.

#### Scenario: Git Source Flow
Given the user selects "Git" source
Then the system prompts for "Git Repository URL"
And the system prompts for "Chart Path"
And the system prompts for "Version" (commit/tag/branch)
And the system proposes a new repository with `type: git`

### Requirement: OCI Configuration
The `add` command MUST support adding charts from OCI registries.

#### Scenario: OCI Source Flow
Given the user selects "OCI" source
Then the system prompts for "OCI URL" (e.g. `oci://registry/repo/chart`)
And the system prompts for "Version"
And the system parses the URL to separate Repository URL (`oci://registry/repo`) and Chart Name (`chart`)
And the system proposes a new repository with `type: oci`

### Requirement: Repository Management
The `add` command MUST check if the repository is already configured.

#### Scenario: New Repository
Given the repository URL is not in `vesshelm.yaml`
Then the system proposes adding a new repository entry

#### Scenario: Existing Repository
Given the repository URL is already in `vesshelm.yaml`
Then the system uses the existing repository name

### Requirement: Chart Configuration
The `add` command MUST allow the user to customize chart configuration.

#### Scenario: Set Namespace
Given the system has identified the chart
Then the user is prompted to enter a namespace

### Requirement: Review and Persist
The `add` command MUST display a summary and ask for confirmation before writing.

#### Scenario: User confirms
Given the user reviews the proposed changes
And the user answers "Yes" to "Add to config?"
Then the system updates `vesshelm.yaml`
And the system runs validation

### Requirement: Coherent Summary
The `add` command MUST display a summary that matches the visual style of existing commands, specifically `sync`.

#### Scenario: Visual Style and Alignment
Given the command lists added items
Then the status tag `[NEW]` is displayed
And the status tag is aligned with `[OK]` tags from other commands (e.g. using similar padding)
And the repository/chart name follows the tag
### Requirement: Code Quality
The `add` command implementation MUST be modular and testable.

#### Scenario: Unit Tests
Given the `add` command logic is refactored
Then unit tests exist for `ChartSource` implementations (parsing logic)
And unit tests exist for `ConfigUpdater` (file modification logic)


### Requirement: Minimal Config Update
The `add` command MUST update `vesshelm.yaml` without modifying existing content (comments, ordering).

#### Scenario: Update Config
Given the user confirms adding a new chart
Then the system appends the new entry to `vesshelm.yaml`
And the system DOES NOT re-serialize the entire file (preserving comments)

### Requirement: Concise Configuration
The `add` command MUST NOT write default or null values to `vesshelm.yaml`.

#### Scenario: Generate YAML
Given a new chart entry is generated
Then null fields (like `dest`, `helm_args`) are omitted from the YAML
And boolean fields with default values are omitted
