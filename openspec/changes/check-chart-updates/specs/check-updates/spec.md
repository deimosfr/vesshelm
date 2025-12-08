# Check Updates Capability

## ADDED Requirements

### Requirement: Check for newer chart versions
The system MUST be able to check remote repositories for newer versions of defined charts.

#### Scenario: Check for updates
Given a `vesshelm.yaml` with outdated charts
When I run `vesshelm check-updates`
Then it should list charts that have newer versions available
And it should show the current and latest version

### Requirement: Handle non-remote charts
The system MUST explicitly ignore local or git-based charts during update checks.

#### Scenario: Ignore local/git charts
Given a `vesshelm.yaml` with local or git charts
When I run `vesshelm check-updates`
Then it should explicitly state that these charts are ignored/skipped for update checking

### Requirement: Apply updates to configuration
The system MUST optionally update the configuration file with the found newer versions.

#### Scenario: Apply updates
Given a `vesshelm.yaml` with outdated charts
When I run `vesshelm check-updates --apply`
Then it should update `vesshelm.yaml` with the new versions
And it should output which charts were updated

### Requirement: Check specific charts
The system MUST allow checking updates for only a subset of specified charts.

#### Scenario: Filter charts
Given a `vesshelm.yaml` with multiple charts
When I run `vesshelm check-updates --only chart1`
Then it should only check/update `chart1`

### Requirement: SemVer Compliance
The system MUST use Semantic Versioning for comparing chart versions.

#### Scenario: Pre-release handling
Given a chart with version `1.19.0-pre.3` installed
And the repository has latest stable version `1.18.4`
When I run `vesshelm check-updates`
Then it should NOT suggest updating to `1.18.4`
And it should consider the installed version as up-to-date (or newer)

#### Scenario: Prefix handling
Given a chart with version `v1.19.2` installed
And the repository has version `1.19.2`
When I run `vesshelm check-updates`
Then it should consider them equal
And it should report "Up to date"

### Requirement: Non-destructive Configuration Updates
The system MUST update the `vesshelm.yaml` file non-destructively.
It MUST NOT re-order fields, remove comments, or add null values for missing optional fields.
It SHOULD only modify the version string of the target chart.

#### Scenario: Preserve comments and formatting
Given a `vesshelm.yaml` with comments and custom field ordering
And an outdated chart
When I run `vesshelm check-updates --apply`
Then the file content should be identical except for the updated version string
And comments should be preserved
And no new `null` fields should be added
