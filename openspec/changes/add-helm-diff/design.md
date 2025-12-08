# Design: Helm Diff Integration

## Behavior Logic

### Configuration
`vesshelm.yaml`:
```yaml
helm:
  args: "..."
  diff_args: "upgrade --allow-unreleased {{ name }} {{ destination }}/{{ name }} -n {{ namespace }}"
  diff_enabled: true # Default false? User wants "a boolean in helm config with diff enabled by default" -> default true?
```
*Clarification*: User said "I'd like to have a boolean in helm config with diff enabled by default". This implies if the user explicitly sets it, or if missing? Usually defaults are explicitly set in code. I will assume if the key is missing from config, it defaults to `false` for backward compatibility, but the user WANTS it enabled by default. I'll make the default `true` in code or config init?
Actually, `serde` default is often false for bools. I'll add `#[serde(default = "default_true")]`.

### Deploy Logic
1. **Dry Run Mode** (`vesshelm deploy --dry-run`):
   - Iterate charts.
   - Run `helm diff ...` (using `diff_args`).
   - DO NOT run standard `helm upgrade`.

2. **Standard Deploy** (`vesshelm deploy`):
   - Iterate charts.
   - If `diff_enabled`:
     - Run `helm diff ...`.
     - Then run `helm upgrade ...`.
   - Else:
     - Run `helm upgrade ...`.

### Argument Construction
`diff_args` will support the same variable interpolation as `args`.
It likely needs to mirror the structure of `helm upgrade` but prefixed with `diff`.
Example default `diff_args`: `"upgrade --allow-unreleased {{ name }} {{ destination }}/{{ name }} --color --context 3"` (standard diff flags).

## Dependencies
- `helm` binary must have `diff` plugin installed.
