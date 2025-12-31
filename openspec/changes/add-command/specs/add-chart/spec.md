# Spec: Add Chart

**capability-id**: `add-chart`

## ADDED Requirements

### Requirement: Interactive Wizard
The `add` command MUST provide an interactive wizard to collect chart information.

#### Scenario: User starts wizard
Given the user runs `vesshelm add`
Then the system prompts for an Artifact Hub URL

### Requirement: Artifact Hub Parsing
The `add` command MUST parse the provided Artifact Hub URL to extract chart details.

#### Scenario: Valid URL provided
Given the user enters `https://artifacthub.io/packages/helm/dnsmasq-k8s/dnsmasq-k8s`
Then the system detects:
  - Repository URL: `https://deimosfr.github.io/dnsmasq-k8s/`
  - Chart Name: `dnsmasq-k8s`
  - Version: `1.4.1`

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
And secondary information in parentheses (e.g. version, url) is displayed in **dimmed** color (grey)

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
