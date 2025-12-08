# Check Updates Capability

## ADDED Requirements

### Requirement: Validate Configuration Syntax

The system MUST validate the structure of the `vesshelm.yaml` configuration file and report specific syntax errors.

#### Scenario: Malformed Repository Indentation

Given a `vesshelm.yaml` with incorrect indentation for repository `url`
When I run `vesshelm check-updates`
Then it should fail with a clear error message indicating a parsing or structure error
And it should not panic or silently ignore the repository
