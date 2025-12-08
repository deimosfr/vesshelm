# Proposal: Ignore commented lines during template rendering

## Goal

<!-- What is the primary objective? -->
Modify the template rendering logic (using `minijinja`) to ignore lines that are commented out in YAML.

## Motivation

<!-- Why this change? What problem does this solve? -->
Currently, `vesshelm` uses `minijinja` with strict undefined behavior. If a YAML values file contains a comment that includes Jinja-like syntax (e.g., `# This is for {{ undefined_var }}`), rendering will fail if the variable is not defined in the context. Users expect comments to be truly ignored and not cause rendering errors.

## What Changes

<!-- Describe what will change. Be specific about new capabilities, modifications, or removals. -->
The `render_template` function in `src/util/variables.rs` will be updated to pre-process the input content. Lines that consist only of comments (optionally preceded by whitespace) will be effectively ignored by Jinja by replacing them with empty lines before rendering. This ensures that:
- Commnets containing `{{ ... }}` placeholders don't trigger "undefined variable" errors.
- Line numbers in error messages (if any other part fails) are preserved because we replace with empty lines rather than deleting them.
- YAML parsing after rendering remains valid.

## Capabilities

### New Capabilities
<!-- Capabilities being introduced. Replace <name> with kebab-case identifier (e.g., user-auth, data-export, api-rate-limiting). Each creates specs/<name>/spec.md -->
- `template-comments-ignoring`: Ignore lines that are YAML comments during the Jinja template rendering process to avoid evaluation of placeholders within comments.

### Modified Capabilities
<!-- Existing capabilities whose REQUIREMENTS are changing (not just implementation).
     Only list here if spec-level behavior changes. Each needs a delta spec file.
     Use existing spec names from openspec/specs/. Leave empty if no requirement changes. -->
- None

## Impact

<!-- Affected code, APIs, dependencies, systems -->
- `src/util/variables.rs`: Logic in `render_template` for pre-processing.
- All commands that use `load_variables` or `render_values_file` (e.g., `deploy`, `ai-update`).
