# Proposal: Allow Disabling Deploy Diff

## Summary
Add a `--no-diff` CLI flag to the `deploy` command. This allows users to forcibly disable the interactive diff view, even if it is enabled in `vesshelm.yaml`.

## Motivation
Currently, users can enable diffs globally in the configuration. However, there are scenarios (e.g., CI pipelines where a previous step already validated changes, or rapid local iteration) where the user wants to skip the diff calculation and interaction step without modifying the configuration file.

## Design Details
- Introduce a boolean flag `--no-diff` to `vesshelm deploy`.
- When set, this flag overrides `vesshelm.yaml`'s `diff_enabled` setting (specifically, it forces diffs off).
- The flag should be ignored or handled gracefully if `dry_run` is active (since dry-run is effectively a diff/plan).

## Alternatives Considered
- Using `VESSHELM_NO_DIFF` env var: Less discoverable than a CLI flag.
- `--diff=false`: `clap` boolean flags are idiomatic as `--no-diff`.

## Complexe Scenarios & Fixes
- **CLI Conflict**: Ensure `dry-run` and `no-diff` conflict correctly (Test: `test_deploy_no_diff_conflict_dry_run` failed on message mismatch).
- **Struct Initialization**: Several tests manually construct `DeployArgs`. These must be updated to include `no_diff: false`.
- **Confirmation Logic**: When `no-diff` is used, the interactive diff preview is skipped. We must clarify/verify if the confirmation prompt is also skipped (implied by current code analysis) or if it should persist.
    - *Decision*: Consistent with `diff_enabled: false`, if no diff is shown, no confirmation is prompted unless we decoupling them. For this change, we maintain current behavior: `should_diff` block contains confirmation. If `no_diff` skips `should_diff`, it skips confirmation. This is "blind deploy" mode, essentially.
    - Users can use `--no-interactive` to explicitly skip prompts, but `--no-diff` forcing skip of prompt is a side effect.
    - *Requirement*: Add a test case verifying `deploy --no-diff` does NOT proceed if interactive mode is on? No, `Confirm::new()` is inside. So it acts like `no-interactive` effectively?
    - Wait, if `should_diff` is false, `execute_helm_command` is called directly.
    - `execute_helm_command` just runs.
    - So `vesshelm deploy --no-diff` in interactive mode acts like `vesshelm deploy --no-diff --no-interactive`?
    - If so, this is a **significant behavior change** or "surprise".
    - Scenario: User wants to skip just the diff step (because it fails or is slow) but still want to confirm "Ready to deploy release X to Y?".
    - Currently, they can't.
    - Proposal: We should probably move confirmation OUT of `should_diff` if we want to allow skipping diff but keeping confirmation.
    - BUT, for `openspec-apply`, we focus on *requested* outcome "add option to disable the diff".
    - The user didn't ask "change confirmation logic".
    - So we document this behavior as "Known Behavior" or "Constraint" and test it.
    - Test: `deploy --no-diff` should succeed without hanging on stdin (which implies no prompt).
