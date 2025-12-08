# Change: Add README

## Why
The project currently lacks a `README.md`. Users need documentation to understand what the tool does, how to use it, and see concrete examples of configuration and commands.

## What Changes
- Create a `README.md` file in the project root.
- The README will include:
  - **Introduction**: Description of Vesshelm as an alternative to helm-freeze/helmfile.
  - **Installation**: (Placeholder or build from source instructions).
  - **Usage**:
    - `vesshelm init`: Initialize configuration.
    - `vesshelm sync`: Download charts.
    - `vesshelm deploy`: Deploy charts (with dependencies and diff support).
    - `vesshelm graph`: Visualize dependencies.
    - `vesshelm validate`: Validate config.
  - **Configuration**:
    - Explanation of `vesshelm.yaml` structure (repositories, charts, destinations, helm config).
    - Example configuration showing local charts, git repos, overrides, and dependencies.

## Impact
- **Documentation**: New file added. No code changes.
