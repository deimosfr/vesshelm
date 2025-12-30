# Add --apply-sync to check-updates

## Summary
Add a new flag `--apply-sync` (and possibly `apply-and-sync` alias if desired, but sticking to standard kebab-case `--apply-sync` is better) to the `vesshelm check-updates` command. This flag will combine the behavior of `--apply` (updating `vesshelm.yaml` with new versions) and running `vesshelm sync` immediately afterwards.

## Motivation
Users often run `check-updates --apply` and then immediately run `sync` to apply the changes to the cluster. This flag streamlines this common workflow into a single command.

## Nuances
- The `sync` command should only run if `check-updates` actually found and applied updates? Or always?
  - *Decision*: It should probably run only if updates were applied, or maybe always to ensure state is consistent. The user request says "Following to apply, it should run a sync". This implies if apply happens, sync happens. If no updates are found, `apply` doesn't do anything (except print message), so `sync` might arguably be skipped. However, explicitly running `check-updates --apply-sync` suggests the user *wants* to sync the current state.
  - *Refinement*: If `check-updates` finds no updates, `vesshelm.yaml` is unchanged. Running `sync` is harmless but maybe redundant if the user just ran it. However, if the user explicitly asks for `apply-sync`, they probably want to ensure the latest versions are deployed. Let's assume:
    1. If updates found -> Apply -> Sync.
    2. If no updates found -> Skip Sync (or maybe checking user intent: usually "apply" implies "if changes"). Let's stick to "If updates are found and applied, then sync". Reference: "Following to apply, it should run a sync". if apply doesn't "apply" anything, maybe sync shouldn't run.
    - actually `check-updates` logic:
    ```rust
    match (args.apply, updates_found) {
        (true, true) => { ... applies ... }
        (false, true) => { ... suggests apply ... }
        _ => { ... all up to date ... }
    }
    ```
    - So if `updates_found` is false, `apply` logic is skipped. I should probably attach sync to the `(true, true)` branch.
