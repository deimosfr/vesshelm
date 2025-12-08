# Goal
Ensure `sync` command executes charts in alphabetical order.

# Context
The user reported a regression in visual progress reporting, requesting alphabetical execution order. Currently, execution order depends on config file order. Sorting charts alphabetically ensures deterministic execution and output.

# Changes
1.  **Modify `sync` command**: Sort `config.charts` by name before execution.
