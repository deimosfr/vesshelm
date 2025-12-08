# Spec: cli-graph-command

## MODIFIED Requirements

### Requirement: Graph Command
The system MUST provide a command to view the deployment dependency graph. The graph output MUST be colored by level to distinguish hierarchy visually.
The command MUST validate the configuration before generating the graph. If the configuration is invalid, it MUST execution and report errors.

#### Scenario: View Graph
- **WHEN** User runs `vesshelm graph` with a valid configuration.
- **THEN** System displays charts in a tree-like structure showing dependency relationships, and each level of the tree has a distinct color.

#### Scenario: Invalid Configuration
- **WHEN** User runs `vesshelm graph` with an invalid configuration.
- **THEN** System reports validation errors and does not output the graph.
