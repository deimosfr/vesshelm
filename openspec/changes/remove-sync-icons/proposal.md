# Remove Useless Icons from Sync Output

## Goal
Remove the checkmark (✓) and cross (✗) icons from the `vesshelm sync` command output to create a cleaner, more standard log output.

## Motivation
The current output includes emoji icons that are considered redundant and potentially "useless" by users. Removing them simplifies the output and focuses on the text-based status (`[OK]`, `[FAIL]`) which is sufficient and less prone to rendering issues in some terminals.

## External Behaviors
- `vesshelm sync` will no longer display `✓` or `✗` icons next to charts.
- The output format will be `[STATUS] <chart_name> ...`.
- The STATUS prefixes will be aligned to ensure chart names start at the same column.
    - `[OK]  ` (padded)
    - `[SKIP]`
    - `[FAIL]`
- Example:
    ```
    [OK]   chart-a
    [SKIP] chart-b
    [FAIL] chart-c
    ```
