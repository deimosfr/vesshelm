# Design: Only Flag

## CLI Argument
- Use `clap` derive.
- `#[clap(long, value_delimiter = ',')]`
- `only: Option<Vec<String>>`

## Sync Logic
- In `sync.rs` loop:
  - If `args.only` is present, check if `chart.name` is in the list.
  - If not, `continue` (skip).
  - Note: `no_sync` check should probably still apply, or maybe `--only` overrides `no_sync`?
  - *Decision*: `--only` implies explicit intent. If I say `--only foo`, and `foo` has `no_sync: true`, should it sync?
  - Yes, explicit command line argument usually overrides config defaults. However, `no_sync` effectively disables the chart.
  - *Refinement*: Let's stick to simple filtering first. If `no_sync` is high priority, it stays. If `only` allows selecting it, maybe valid.
  - Current `sync.rs` checks `no_sync` first.
  - Proposed flow:
    1. Check `only`. If `only` is set and chart NOT in `only`, SKIP (silent or debug).
    2. Check `no_sync`. If `no_sync` is true, SKIP (log "Skipped (no_sync)").
    3. Proceed.

## Deploy Logic
- In `deploy.rs`:
  1. Load ALL charts.
  2. Run `dag::sort_charts(&all_charts)` -> `Result<Vec<&Chart>>`. This ensures the dependency graph is valid and we get the correct topological order.
  3. Filter the `sorted_charts` vector.
     - Keep chart if it matches `args.only`.
  4. Execute deployment for filtered list.
  - This safeguards against graph errors (e.g. missing nodes) by building the full graph, but only executing the requested slice.
