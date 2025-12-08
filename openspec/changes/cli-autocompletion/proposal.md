# CLI Autocompletion

## Background
The user wants to generate shell completion scripts for `vesshelm`. Additionally, they want dynamic completion for chart names (e.g., `vesshelm deploy <TAB>` suggests charts from `vesshelm.yaml`).

## Goal
Implement a `completion` command that outputs shell completion scripts (Bash, Zsh, Fish, PowerShell).
Enable dynamic completion for chart names where possible.

## Changes
- Add `clap_complete` dependency.
- Add `Completion` subcommand to `src/cli/commands/mod.rs`.
- Implement `src/cli/commands/completion.rs` to generate scripts.
- To support *dynamic* completion for chart names:
    - Standard `clap_complete` generates static scripts.
    - For simple dynamic completion, we might need to instruct the user or use features that allow invoking the binary.
    - However, `clap` implies we generate the script.
    - **Refinement**: I will implement the `completion` command first. For dynamic chart completion, I will investigate if we can use a custom value hint or a hidden command `vesshelm list-charts` coupled with a manually enhanced script or if `clap_complete` has hooks.
    - *Correction*: Simpler approach for now is `ValueHint::Other` and maybe relying on standard file completion as fallback, BUT the user asked for config-based completion.
    - I will propose adding the `completion` command and *attempting* to support dynamic completion by utilizing `clap::ValueHint` if suitable, or documenting how it works.
    - **Wait**, `clap_complete` doesn't do dynamic config lookup at shell time dynamically unless the shell script calls the binary.
    - I will stick to: Add `completion` command. Use `clap_complete`. If dynamic completion requires complex manual script editing, I will implement the static part first and see if I can add a `ValueHint` that valid shells might pick up, or add a `list-charts` command that users can hook into if they want advanced completion.
    - Actually, `fish` often parses `--help` or uses generated files.
    - I will promise `completion` command. I will add a requirement for it.

## Plan
1. Add `clap_complete` to `Cargo.toml`.
2. Create `src/cli/commands/completion.rs`.
3. Wiring it up in `main.rs` / `mod.rs`.
