## Why

Reviewing Helm chart updates for security misconfigurations, breaking changes, or stability issues is a manual and error-prone process for SREs. This change introduces `ai-update`, which leverages AI to automatically analyze chart diffs and provide expert SRE insights before finalizing updates.

## What Changes

- **New Command**: `ai-update` which performs:
  1. Pre-flight check: Ensures no uncommitted changes in the local repository (using `git2` library).
  2. Update: Runs the equivalent of `update --apply-sync` to fetch and apply chart updates locally.
  3. Analysis: Captures changes and sends them to a configured AI provider, optimized with `rtk` for token efficiency.
  4. Reporting:
     - AI (acting as an SRE) identifies security risks, stability issues, and breaking changes.
     - **Summary Output**: Clearly distinguishes between "OK" updates (safe to proceed) and those requiring **User Intervention** (due to detected risks).
- **Configuration**: New `ai` section in `vesshelm.yaml` to configure providers (Gemini, Claude, OpenAI, Ollama) and `rtk` settings.
- **Git Integration**: Direct integration using the `git2` Rust library instead of shell commands.
- **Efficiency**: Integration with `rtk` to minimize LLM token usage during diff analysis.
- **Documentation**: Update `README.md` to include usage instructions and configuration details for the new `ai-update` command.

## Capabilities

### New Capabilities
- `ai-update`: Performs automated chart updates followed by AI-powered SRE analysis of the resulting diffs.

### Modified Capabilities
- `config-management`: Extended to support AI provider configuration in `vesshelm.yaml`.

## Impact

- **Dependencies**: Adds support for AI provider APIs (REST/SDKs).
- **Configuration**: `vesshelm.yaml` schema updated with an `ai` section.
- **CLI**: New top-level command `ai-update`.
- **System**: Requires `git` to be available in the PATH.
