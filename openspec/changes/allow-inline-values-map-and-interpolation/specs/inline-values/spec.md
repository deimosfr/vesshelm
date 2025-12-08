# Capability: Inline Values Map and Interpolation

## ADDED Requirements

### Requirement: Support Map Values
The `values` field in the Chart configuration MUST accept a Map (key-value pairs) in addition to the existing Sequence (list of maps).

#### Scenario: User provides values as a Map
- **Given** a `vesshelm.yaml` with a chart defining `values` as a key-value map
- **When** `vesshelm deploy` is run
- **Then** the configuration should load successfully
- **And** the values should be passed to Helm as a values file

#### Scenario: User provides values as a Sequence
- **Given** a `vesshelm.yaml` with a chart defining `values` as a list of maps
- **When** `vesshelm deploy` is run
- **Then** the configuration should load successfully (compatibility preserved)

### Requirement: Support Interpolation in Inline Values
Inline values defined in the `values` field MUST support Jinja2 variable interpolation using the loaded variable context.

#### Scenario: User uses variables in inline values
- **Given** a `vesshelm.yaml` with `values` containing `{{ variable }}`
- **When** `vesshelm deploy` is run using a variable file defining `variable`
- **Then** the rendered values file passed to Helm should contain the interpolated value

#### Scenario: Interpolation with conflicting names
- **Given** a `vesshelm.yaml` with `values` containing `{{ namespace }}` (conflicting with builtin)
- **When** `vesshelm deploy` is run without defining `namespace` variable
- **Then** the interpolation should NOT resolve to the internal function object
- **And** it should behave as an undefined variable (error or empty depending on strictness)
