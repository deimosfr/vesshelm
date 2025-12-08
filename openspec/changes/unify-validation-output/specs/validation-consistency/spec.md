# Validation Consistency

## ADDED Requirements

#### Scenario: Global Validation Consistency
All CLI commands that trigger configuration validation MUST output the same detailed error format.
-   **Given** an invalid configuration (e.g., missing values file)
-   **When** running `vesshelm deploy`
-   **Then** the output contains the specific error details (e.g., "Values file not found: './overrides/missing.yaml'") matching `vesshelm validate`.

#### Scenario: Validation Error Unwrapping
The error formatter MUST be able to find validation errors wrapped in context.
-   **Given** a `ValidationErrors` wrapped in an `anyhow::Error` with context "Configuration validation failed"
-   **When** formatted for display
-   **Then** the underlying validation errors are displayed.
