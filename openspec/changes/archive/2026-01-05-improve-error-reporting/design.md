# Error Reporting Design

## Overview
We will intercept errors at the top level of the application (in `main` or a `run` wrapper) and apply custom formatting logic before printing them to `bstderr`.

## Key Components

### `util::error` Module
A new module responsible for formatting errors.

- **`PrettyError`**: A wrapper or trait extension to format `anyhow::Error`.
- **`Validation Formatter`**: Special handling for `validator::ValidationErrors`. It should unpack the map and print each error on a new line with a bullet point, mapping fields to human-readable names if needed.

## Formatting Rules
- **Header**: "Error:" in **Bold Red**.
- **Body**: Regular text, but key parts (like file paths or values) can be colored if context allows.
- **Causes**: If there's a specific cause (like a validation list), print it cleanly below the header.
- **Examples**:
    ```text
    Error: Configuration validation failed

    - Chart 'nginx': Repository 'bitnami' not found
    - Repository 'stable': URL is invalid
    ```
