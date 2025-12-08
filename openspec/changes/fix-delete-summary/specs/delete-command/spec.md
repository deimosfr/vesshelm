## MODIFIED Requirements

### Requirement: Summary Display
The deletion summary MUST provide clear and accurate information about the chart being filtered.

#### Scenario: Namespace Display
Given a chart is selected for deletion
The summary MUST display the "Namespace" on a dedicated line, separate from the Chart name.

#### Scenario: Path Accuracy
The summary MUST display the full, absolute, or relative path to the chart directory that will be deleted, specifically including the chart's subdirectory (e.g., `charts/my-chart`), not just the parent destination.

#### Scenario: Action Detail
The "Action" summary line MUST explicitly state that the local directory will be deleted, in addition to config/lock updates.
