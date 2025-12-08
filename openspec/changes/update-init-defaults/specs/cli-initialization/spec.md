# Spec: CLI Initialization

## ADDED Requirements

### Requirement: Default Helm Arguments
When initializing a new configuration, Vesshelm MUST generate a `vesshelm.yaml` that includes a default `helm` section with standard arguments.

#### Scenario: Init Generates Default Args
Given a directory without `vesshelm.yaml`,
When `vesshelm init` is executed,
Then the created `vesshelm.yaml` should contain:
```yaml
helm:
  args: "{{ name }} {{ destination }}/{{ name }} -n {{ namespace }} --wait --rollback-on-failure --create-namespace"
```

## MODIFIED Requirements
(No formal modification of existing requirements needed as we are essentially refining the "Default Configuration" requirement which likely wasn't detailed enough in previous specs or didn't exist explicitly in `add-init-command` spec artifacts if they were lost/archived without spec updates. Treating as ADDED for clarity or I can assume an implicit modification if I had the original spec.)
