## ADDED Requirements

### Requirement: AI Re-analyze mode
The system SHALL provide a `--re-analyze` (or `-R`) flag to the `ai-update` command. When this flag is provided, the system SHALL skip the repository clean check and the chart update process, and instead perform an AI impact analysis of the current local configuration and template changes compared to the last commit (HEAD).

#### Scenario: Running re-analyze on dirty repo
- **WHEN** the user runs `vesshelm ai --re-analyze` in a repository with uncommitted changes
- **THEN** the system skips the "git status" check and "chart update" step
- **THEN** the system identifies "old" configuration by reading `vesshelm.yaml` from HEAD
- **THEN** the system generates a template diff between HEAD and the current working directory
- **THEN** the system performs AI analysis on these changes and displays the report

### Requirement: Interactive Dirty Check
The system SHALL detect if the local repository has uncommitted changes before performing a chart update. If changes are detected and the session is interactive, the system SHALL prompt the user for confirmation before proceeding.

#### Scenario: Confirming update on dirty repo
- **WHEN** the user runs `vesshelm ai` and the repository is dirty
- **THEN** the system prompts: "Local repository has uncommitted changes. Syncing may produce a messy diff. Proceed anyway?"
- **WHEN** the user accepts the prompt
- **THEN** the system proceeds with the chart update and subsequent AI analysis

#### Scenario: Aborting update on dirty repo
- **WHEN** the user runs `vesshelm ai`, the repository is dirty, and the user rejects the prompt
- **THEN** the system terminates with an error message and does not perform the update
