# Spec: Variable Interpolation

## ADDED Requirements

### Requirement: Interpolate values files
The application MUST interpolate Jinja2 variables in `values_files` using the context loaded from `variable_files`.

#### Scenario: Basic interpolation
Given a variable file defining `region: us-east-1`
And a chart values file containing `region: {{ region }}`
When I run a deploy or diff command
Then the passed values file to Helm should contain `region: us-east-1`

#### Scenario: Nested variables
Given a variable file defining:
  ```yaml
  cluster:
    name: wandering-star
  ```
And a values file containing `clusterName: {{ cluster.name }}`
When I run a deploy command
Then the rendered values should contain `clusterName: wandering-star`

#### Scenario: Jinja functions
Given I want to use Jinja2 filters or functions
And a values file containing `key: {{ "HELLO" | lower }}`
When I run a deploy command
Then the rendered values should contain `key: hello`

### Requirement: Temporary file handling
The application MUST use temporary files for rendered values during Helm execution and clean them up afterwards.

#### Scenario: Helm Execution
Given a chart with interpolated values
When `helm` is executed
Then the `-f` argument should point to a temporary file containing the rendered YAML
And the temporary file should be removed after execution

#### Scenario: Signal Handling (SIGTERM)
Given the application is running and has created temporary rendered files
When the application receives a `SIGTERM` signal
Then it MUST delete all temporary files before exiting
And it MUST exit with a non-zero status code (if appropriate for the signal handling flow)

### Requirement: Interpolate Local Default Values
For local charts, the application MUST verify if a `values.yaml` exists in the chart directory, render it using the variable context, and pass it as an explicit values file to Helm to ensure variables are interpolated.

#### Scenario: Local chart with variables
Given a local chart with a `values.yaml` containing `key: {{ variable }}`
And a global variable definitions file with `variable: value`
When I run a deploy or diff command
Then the `values.yaml` should be rendered
And passed to Helm as a temporary file (e.g. `-f /tmp/rendered.yaml`)
And the deployed release should have `key: value`


### Requirement: Comprehensive Testing
The variable interpolation feature MUST be covered by integration tests to ensure reliability across different scenarios.

#### Scenario: Integration Test - Local Chart Interpolation
Given a local chart with `values.yaml` containing variables
And a configured variable file
When `vesshelm deploy` is run
Then the test should verify that the correct values are passed to Helm (by inspecting dry-run output or mock).

#### Scenario: Integration Test - Missing Variables
Given a chart requesting a variable that is not defined
When `vesshelm deploy` is run
Then the deployment should fail with a descriptive error
