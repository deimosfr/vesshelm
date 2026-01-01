## ADDED Requirements

### Requirement: CLI Interface
The CLI MUST provide a user-friendly interface for deleting charts, supporting both direct arguments and interactive selection.

#### Scenario: Running the delete command
Given the user is in the project root
When the user runs `vesshelm delete [CHART_NAME]`
Then the CLI should initiate the deletion process for the specified chart.

#### Scenario: No Arguments
Given the user runs `vesshelm delete` without arguments
Then the command MUST list all available charts from `vesshelm.yaml`.
And the list MUST be sorted alphabetically by chart name.
And allow the user to fuzzy-select one.

### Requirement: Validation
The system MUST ensure that chart deletions do not break existing dependencies.

#### Scenario: Blocking deletion due to dependencies
Given a chart "base-chart"
And a chart "app-chart" that depends on "base-chart"
When the user attempts to delete "base-chart"
Then the CLI should abort the operation
And display a message listing "app-chart" as a dependent.

### Requirement: Summary and Confirmation
The CLI MUST inform the user about the impact of the deletion and require explicit confirmation.

#### Scenario: displaying impact summary
Given a chart "my-chart" with no dependents
When the user proceeds to cleanup
Then the CLI should display a summary including:
  - The chart name.
  - The path of the directory to be deleted (aligned nicely).
  - Use text-based status indicators `[OK]`, `[FAIL]` similar to `sync` command instead of emojis.
  - Confirmation that it will be removed from configuration and lockfile.
  - (If applicable) The repository that will be removed.

#### Scenario: User confirmation
After displaying the summary
The CLI must ask "Do you want to continue?" (default: No).
If the user declines, no changes are applied.

### Requirement: Execution
The system MUST perform a comprehensive cleanup of the chart from the filesystem and all configuration files.

#### Scenario: Cleanup
Given the user confirms deletion of "my-chart"
Then the system must:
  1. Delete the local directory for "my-chart".
  2. Remove the "my-chart" entry from `vesshelm.yaml`.
  3. Remove the "my-chart" entry from `vesshelm.lock`.
  4. Save both files.

#### Scenario: Robust Config Removal
Given `vesshelm.yaml` contains comments, section headers (lines starting with `#` at root level), or extra whitespace between chart entries
When a chart is deleted
Then the system MUST correctly identify and remove the chart block
And MUST NOT consume comments or section headers that are separated from the item by at least one empty line.
And MUST process list items correctly even if they are followed immediately by a comment block if that block is clearly a section header (e.g. root level).

#### Scenario: Repository Cleanup
Given a repository "my-repo" used ONLY by "my-chart"
When "my-chart" is deleted
Then "my-repo" should also be removed from `vesshelm.yaml`.

#### Scenario: Repository Preservation
Given a repository "shared-repo" used by "chart-A" and "chart-B"
When "chart-A" is deleted
Then "shared-repo" MUST remain in `vesshelm.yaml` because "chart-B" still uses it.

### Requirement: Visual Consistency
The command output MUST match the visual style of `vesshelm sync`.

#### Scenario: Icons and Formatting
Given the user runs the command
Then the output MUST use `[OK]` (green), `[FAIL]` (red), `[WARN]` (yellow) text statuses instead of emojis.
And the summary table MUST be properly aligned with consistent padding (e.g. 25 characters) to accommodate long labels like "Uninstall release".
