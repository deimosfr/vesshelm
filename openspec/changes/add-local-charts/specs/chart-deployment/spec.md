# Spec: Chart Deployment

## ADDED Requirements

### Requirement: Global Helm Arguments
The `vesshelm.yaml` configuration MUST support a `helm` section with an `args` field to define the default arguments for the `helm` command.

#### Scenario: Default Deployment
Given a `vesshelm.yaml` with:
```yaml
helm:
  args: "upgrade --install {{ name }} {{ destination }}/{{ name }} -n {{ namespace }}"
charts:
  - name: my-chart
    version: 1.0.0
```
When running `vesshelm deploy`,
Then it should execute `helm upgrade --install my-chart ./charts/my-chart -n default` (assuming default destination and namespace).

### Requirement: Chart Overrides
The chart configuration MUST support `helm_args_override` and `helm_args_append` to customize deployment for specific charts.

#### Scenario: Appending Arguments
Given a chart with `helm_args_append: "--create-namespace"`,
When running `vesshelm deploy`,
Then the `--create-namespace` flag should be appended to the generated helm command.

#### Scenario: Overriding Arguments
Given a chart with `helm_args_override: "install {{ name }} . --wait"`,
When running `vesshelm deploy`,
Then the generated command should completely replace the global args with the overridden string.

### Requirement: Variable Interpolation
The arguments string MUST support interpolation of chart configuration values.

#### Scenario: Interpolating Variables
Given `helm.args` containing `{{ name }}`, `{{ destination }}`, and `{{ namespace }}`,
When `vesshelm deploy` constructs the command,
Then these placeholders should be replaced with the actual chart name, destination path, and namespace.
