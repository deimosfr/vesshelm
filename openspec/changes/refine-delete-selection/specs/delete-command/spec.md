## MODIFIED Requirements

### Requirement: CLI Interface
The CLI MUST provide a user-friendly interface for deleting charts, supporting both direct arguments, interactive selection, and namespace disambiguation.

#### Scenario: Ambiguous Chart Name
Given the user runs `vesshelm delete my-chart`
And there are two charts named "my-chart" in namespaces "prod" and "dev"
Then the CLI MUST prompt the user to select the desired namespace.

#### Scenario: Unique Chart Name
Given the user runs `vesshelm delete unique-chart`
And there is only one chart named "unique-chart"
Then the CLI SHOULD proceed without asking for namespace.

### Requirement: Configuration Preservation
The system MUST update `vesshelm.yaml` non-destructively, removing only the deleted chart's entry while preserving all other content, comments, and formatting.

#### Scenario: In-place Update
Given a `vesshelm.yaml` with comments and custom formatting
When a chart is deleted
Then the file content MUST remain identical except for the removal of the specific chart block.
