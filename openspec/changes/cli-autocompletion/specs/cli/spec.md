# CLI Autocompletion

## ADDED Requirements

### Requirement: Generate shell completion scripts
The CLI MUST provide a `completion` command that generates shell completion scripts for supported shells (Bash, Zsh, Fish, PowerShell).
- The command MUST accept a `--shell` or positional argument to specify the target shell.
- The command MUST output the script to stdout.

#### Scenario: Generate bash completion
When the user runs `vesshelm completion bash`
Then the command outputs the Bash completion script.

#### Scenario: Generate zsh completion
When the user runs `vesshelm completion zsh`
Then the command outputs the Zsh completion script.

### Requirement: Chart name suggestion
The CLI completion scripts MUST suggest available chart names from `vesshelm.yaml` when completing arguments for `deploy`, `sync`, `check-updates`, and `uninstall`.

#### Scenario: Complete chart for deploy
Given a `vesshelm.yaml` with charts `foo` and `bar`
When the user types `vesshelm deploy ` and presses Tab
Then `foo` and `bar` are suggested.
