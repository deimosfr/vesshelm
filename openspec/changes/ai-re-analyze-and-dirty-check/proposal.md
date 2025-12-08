# Proposal: AI Re-analyze and Relaxed Dirty Check

## Why
When running `vesshelm ai`, the tool currently enforces a clean git repository. This can be restrictive if:
1. The user has unrelated changes but wants to perform an AI-powered chart update.
2. The user has already run `vesshelm ai`, received AI warnings, made manual adjustments to the configuration, and wants to re-run the analysis to verify their fixes.

In the second scenario, the repo is *guaranteed* to be dirty (because of the initial version bump and the manual fixes), which means the user must either commit their "experimental" fixes or be blocked from re-analyzing. Adding a way to re-analyze existing changes significantly improves the developer feedback loop.

## What Changes
- **Relaxed Pre-flight Check**: Instead of failing immediately when the repo is dirty, the tool will now prompt the user for confirmation (falling back to failure in non-interactive modes).
- **Re-analyze Mode**: A new `--re-analyze` (shorthand `-R`) flag for the `ai-update` command. This mode skips chart syncing and version checking, instead focusing solely on the current local changes compared to the last commit (HEAD).

## Capabilities

### New Capabilities
- `ai-re-analyze`: Allows triggering an AI analysis of current configuration changes and template diffs without performing a chart update. This uses the current workspace state compared against HEAD.
- `ai-update-dirty-handling`: Modifies the core `ai-update` workflow to handle dirty repositories gracefully via user interaction or forced bypass.

## Impact
- **Affected Commands**: `vesshelm ai` (`ai-update`).
- **New Command Arguments**: `--re-analyze` / `-R`.
- **User Experience**: Faster iteration for chart updates; less terminal friction.
- **Git Integration**: Requires new utility to read file content from HEAD to accurately identify "old" vs "new" configuration in re-analyze mode.
