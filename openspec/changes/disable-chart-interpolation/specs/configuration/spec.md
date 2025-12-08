# Configuration Specs

## ADDED Requirements

### Requirement: Chart configuration supports `no_interpolation`
The `Chart` configuration object MUST support a `no_interpolation` boolean field that defaults to `false`.

#### Scenario: Default behavior
GIVEN a chart configuration without `no_interpolation` field
WHEN the configuration is loaded
THEN `no_interpolation` is `false`

#### Scenario: Explicitly disabled
GIVEN a chart configuration with `no_interpolation: true`
WHEN the configuration is loaded
THEN `no_interpolation` is `true`

### Requirement: Interpolation is skipped when `no_interpolation` is true
When deploying a chart with `no_interpolation: true`, the values files MUST NOT be processed by the template engine, even if variables are available.

#### Scenario: Interpolation disabled in deploy
GIVEN a chart with `no_interpolation: true`
AND a values file containing `key: {{ value }}`
AND a variable context where `value` is defined
WHEN the chart is deployed
THEN the values file is passed to Helm as-is (preserving `{{ value }}`)
AND the template engine is NOT invoked for this file
