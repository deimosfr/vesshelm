## 1. Configuration and Dependencies

- [x] 1.1 Add `git2` and `rtk` (and any related dependencies) to `Cargo.toml`
- [x] 1.2 Add `AIConfig` struct and related enums (including `RTKConfig`) to `src/config.rs`
- [x] 1.3 Update `vesshelm.yaml` deserialization to include the `ai` and `rtk` sections
- [x] 1.4 Add validation for AI and RTK configuration in `src/config.rs`

## 2. AI Client and RTK Integration

- [x] 2.1 Create `src/clients/ai/mod.rs` and define the `AIProvider` trait
- [x] 2.2 Implement the `Gemini`, `Claude`, `OpenAI`, and `Ollama` providers
- [x] 2.3 Integrate `rtk` to optimize the payload sent to AI providers

## 3. Git Integration with git2

- [x] 3.1 Create `src/util/git.rs` using the `git2` crate
- [x] 3.2 Implement `is_dirty()` using `git2::Repository::statuses()`
- [x] 3.3 Implement change capture (diff) after update using `git2`

## 4. CLI Command and Summarization

- [x] 4.1 Add `ai-update` command to the CLI structure in `src/cli/commands/mod.rs`
- [x] 4.2 Implement `src/cli/commands/ai_update.rs` runner
- [x] 4.3 Implement pre-flight dirty check logic using `git2`
- [x] 4.4 Implement `update --apply-sync` call within `ai-update`
- [x] 4.5 Implement SRE analysis logic: capture changes, call optimized AI via `rtk`
- [x] 4.6 Implement the summary output: categorizing results into "OK" and "User Intervention" sections

## 5. Testing and Validation

- [x] 5.1 Add unit tests for AI and RTK configuration parsing
- [x] 5.2 Add unit tests for `git2` integration logic
- [x] 5.3 Add integration test for the full `ai-update` workflow with summary output

## 6. Documentation

- [x] 6.1 Update `README.md` with `ai-update` command and configuration details
