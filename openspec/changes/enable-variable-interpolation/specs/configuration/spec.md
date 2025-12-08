# Spec: Configuration Update

## ADDED Requirements

### Requirement: Support variable_files configuration
The application MUST support a `variable_files` list in the root configuration to define global variable sources.

#### Scenario: Defining global variables
Given a `vesshelm.yaml` file
When I add `variable_files` with a list of paths
Then the application successfully loads the configuration
And validates that the files exist

#### Scenario: Variable files paths
Given `variable_files` containing relative paths
When the configuration is loaded
Then the paths should be resolved relative to the configuration file location

### Requirement: Explicit Parsing Error
The application MUST report explicit errors when configuration parsing fails, indicating the specific cause (e.g., missing field, invalid type).

#### Scenario: Missing configuration field
Given a `vesshelm.yaml` with a missing required field
When the application attempts to load the configuration
Then it should fail with an error message detailing the missing field
And NOT just "Failed to parse configuration file"

### Requirement: Friendly Validation Error
The application MUST format validation errors in a user-friendly way, removing internal implementation details like `Field '__all__'`.

#### Scenario: Validation failure
Given a configuration with validation errors (e.g. missing variable file)
When the application attempts to validate the configuration
Then it should display "Error: variable_file_not_found" or similar clean message
And NOT "Error: Field '__all__' failed validation: ..."
