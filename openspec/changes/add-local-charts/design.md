# Design: Local Chart Deployment

## Configuration Schema

### Global Config
A new `helm` section will be added to the root configuration:

```yaml
helm:
  # Default arguments for helm deployment
  args: "{{ name }} {{ destination }}/{{ name }} -n {{ namespace }} --wait --rollback-on-failure"
```

### Chart Config
The `Chart` configuration will be extended with:
- `comment` (string, optional): For documentation or metadata (e.g., source URL).
- `helm_args_append` (string, optional): Arguments to append to the default or overridden args.
- `helm_args_override` (string, optional): Completely replaces the global `helm.args`.
- `destination_override` (string, optional): Overrides the calculated destination path.
- `values` (list of objects, optional): Inline values (e.g., `[{key: value}]`). *(Note: User mentioned this in example, needing verification if it maps to `--set`)*.
- `depends` (list of strings, optional): Dependency ordering (to be implemented if needed for deploy order, explicitly mentioned in user example).

## Variable Interpolation
The argument strings must support placeholders in the format `{{ variable }}`.
Supported variables:
- `name`: Chart name.
- `namespace`: Chart namespace.
- `destination`: The directory where the chart is located.
- `version`: Chart version.

## Command Execution
The `deploy` command will:
1. Parse the configuration.
2. Filter charts if `helm_args_override` or `no_sync` (if applicable to deploy) dictates logic, though `no_sync` usually applies to download.
3. Determine deployment order (respecting `depends` if implemented).
4. For each chart:
    - Construct the `helm` command string.
    - Interpolate variables.
    - Execute the command using `std::process::Command` (or similar).
    - Stream output to stdout/stderr.

## Dependencies
- `handlebars` or `tera` or custom regex for interpolation.
- `topological-sort` (if `depends` is implemented for strict ordering).
