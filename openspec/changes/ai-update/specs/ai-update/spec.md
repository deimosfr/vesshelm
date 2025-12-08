## ADDED Requirements

### Requirement: ai-update command
The system SHALL provide an `ai-update` command that automates the update process and performs AI-driven SRE analysis of the resulting changes.

#### Scenario: Successful AI update with summary
- **WHEN** user runs `vesshelm ai-update`
- **THEN** system checks for uncommitted files using `git2`, runs update --apply-sync, analyzes changes using `rtk` and AI, and presents a summary of "OK" vs "User Intervention" updates.

#### Scenario: Blocked by uncommitted files
- **WHEN** user runs `vesshelm ai-update` AND there are uncommitted files in the workspace (detected via `git2`)
- **THEN** system SHALL abort with an error message and NOT perform the update

### Requirement: AI Configuration
The `vesshelm.yaml` configuration SHALL include an `ai` section to define AI provider settings and `rtk` optimization parameters.

#### Scenario: Valid AI and RTK configuration
- **WHEN** `vesshelm.yaml` contains a valid `ai` section with provider, API key, and `rtk` settings
- **THEN** `vesshelm ai-update` SHALL use the configured provider and `rtk` for efficient analysis

#### Scenario: Missing AI configuration
- **WHEN** user runs `vesshelm ai-update` AND the `ai` section is missing or invalid in `vesshelm.yaml`
- **THEN** system SHALL abort with a helpful error message explaining how to configure the AI provider

### Requirement: SRE Analysis Summary
The AI SHALL act as a Site Reliability Engineer (SRE) and categorize updates into a clear summary.

#### Scenario: Categorizing updates as OK
- **WHEN** the chart diff is safe (no security risks, no breaking changes)
- **THEN** AI SHALL mark the update as "OK" in the final summary

#### Scenario: Categorizing updates as User Intervention
- **WHEN** the chart diff introduces a security risk or stability issue
- **THEN** AI SHALL mark the update as requiring "User Intervention" and provide detailed reasoning in the output

### Requirement: README Documentation
The `README.md` file MUST be updated to include the new `ai-update` command and its configuration.

#### Scenario: Documentation updated
- **WHEN** user looks at `README.md`
- **THEN** they SHOULD see a clear section explaining how to use `vesshelm ai-update` and how to configure the `ai` section in `vesshelm.yaml`.
