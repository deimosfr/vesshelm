# Improve Error Reporting

## Summary
Improve the CLI error reporting to be more user-friendly, "fancy", and clear. Specifically target configuration validation errors to replace technical dump formats with readable, actionable feedback.

## Motivation
Current error output, especially from the `validator` crate, is technical and difficult for users to parse (e.g., `__all__: Validation error: chart_repo_not_found [{}]`). Users need clear, formatted, and color-coded error messages to quickly identify and fix issues.

## Proposed Solution
- Implement a custom error reporting mechanism (or intercept errors in `main.rs`) that detects specific error types (like `ValidationErrors`) and formats them nicely.
- Use the `colored` crate to style "Error:" headers and content.
- Ensure all command failures follow this consistent, accessible style.
