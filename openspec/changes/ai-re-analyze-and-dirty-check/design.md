# Design: AI Re-analyze and Relaxed Dirty Check

## Context
Currently, `ai-update` (aka `ai`) enforces a clean git repository. This prevents running the command when there are unrelated local changes, even if safe. More importantly, it prevents a "re-analyze" workflow where the user manually fixes configuration based on AI feedback and wants to check the results again before committing.

## Goals / Non-Goals

**Goals:**
- Provide a way to re-run AI analysis without re-syncing or version checking.
- Allow running `ai-update` on dirty repositories if the user explicitly confirms.
- Accurately identify "original" vs "modified" configuration even when the current configuration on disk has already been modified.

**Non-Goals:**
- Automated staging or committing of changes.
- Multi-branch diffing; only HEAD vs Working Tree is considered.

## Decisions

### 1. New Flag: `--re-analyze` / `-R`
This flag for `ai-update` will:
- Skip the "Pre-flight Git Check" (dirty repo validation).
- Skip the "Perform Update" (syncing charts and bumping versions).
- Load the configuration from the last commit (HEAD) as the base (old config).
- Load the current configuration from disk as the target (new config).
- Extract the diff of rendered Kubernetes templates from HEAD to the current state.

**Rationale:** This flag allows a tight feedback loop where users can iterate on their configuration and immediately see if the AI [OK] status changes.

### 2. Interactive Dirty Check
Instead of a hard failure in `ai-update`, the check will become:
```rust
if git.is_dirty()? {
    if !interaction.confirm("Local repository has uncommitted changes. Syncing may produce a messy diff. Proceed?", false)? {
         return Err(VesshelmError::CommandFailed("User aborted due to dirty repository".to_string()));
    }
}
```
**Rationale:** Standard CLI patterns usually allow overriding safety checks, either via `-y` (implied by non-interactive) or interactive confirmation.

### 3. Git Utility Expansion
Add a method `GitUtil::read_file_from_head(path: &Path)` to retrieve file contents from the last commit. This is crucial for "re-analyze" mode to correctly identify what changed in `vesshelm.yaml` since the last commit.

### 4. Diff Strategy in Re-analyze Mode
Modify `GitUtil::get_diff()` to optionally take a base reference (like HEAD). If re-analyzing, we want the diff from HEAD to Workspace. The current implementation uses `diff_index_to_workdir`, which skips staged changes. It should be updated to `diff_tree_to_workdir(HEAD)` for a more comprehensive view.

## Risks / Trade-offs

- **[Risk]** User runs `ai-update` on a dirty repo and gets a diff containing unrelated changes, confusing the AI.
  - **Mitigation** The AI analysis is focused on charts identified as "changed" in the config. The confirmation prompt explicitly warns about "messy diffs".
- **[Risk]** `vesshelm.yaml` is not at the root of the git repository.
  - **Mitigation** `GitUtil` will resolve the relative path of the configuration file within the discovered git repository.
