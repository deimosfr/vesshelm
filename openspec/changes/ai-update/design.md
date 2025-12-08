## Context

Vesshelm currently allows updating charts, but lacks automated safety analysis of the changes. This design introduces an AI-powered analysis step that acts as an SRE reviewer for chart diffs.

## Goals / Non-Goals

**Goals:**
- Provide an `ai-update` command.
- Support multiple AI providers (Gemini, Claude, OpenAI, Ollama).
- Automate pre-flight checks using the `git2` library.
- Automate change analysis with `rtk` for token efficiency.
- Report security and stability risks with clear "OK" vs "User Intervention" summaries.

**Non-Goals:**
- Automatically fixing issues identified by AI.
- Supporting non-git environments for `ai-update`.
- Providing a GUI for the analysis results.

## Decisions

### 1. AI Provider Abstraction
**Decision**: Use a trait-based approach for AI providers, integrated with `rtk`.
**Rationale**: Allows easy addition of new providers and leverages `rtk` to minimize the number of tokens sent to LLMs, reducing costs and improving speed.

### 2. Git Integration
**Decision**: Use the `git2` Rust crate for all repository operations.
**Rationale**: Provides deep, programmatic access to Git without relying on external binaries, ensuring consistent behavior across different environments.
**Alternatives**: Calling the `git` binary (previously considered, but rejected for robustness).

### 3. SRE Prompt Engineering
**Decision**: Use a system-level prompt that instructs the AI to act as a Senior SRE, focusing on categorizing updates as "OK" or requiring "User Intervention".
**Rationale**: Provides clear, actionable feedback to the user and ensures high-quality security and stability reviews.

### 4. Configuration
**Decision**: Add a top-level `ai` section to `vesshelm.yaml` with provider-specific settings and `rtk` optimization toggles.
**Rationale**: Keeps AI and token optimization settings centralized.

## Risks / Trade-offs

- **[Risk]** → **AI Hallucinations**: AI might report false positives or miss subtle issues.
  - **Mitigation**: Clear disclaimer in output and focus on AI as a "reviewer" rather than an "authority".
- **[Risk]** → **API Costs/Latency**: Running AI analysis on large diffs could be slow or expensive.
  - **Mitigation**: Truncate very large diffs or warn the user before sending large amounts of data. `rtk` helps mitigate token-related costs.
- **[Risk]** → **Sensitive Data Leak**: Diff might contain secrets.
  - **Mitigation**: Advise users to use SOPS (already supported in Vesshelm) and warn about sending unencrypted diffs to cloud providers.
