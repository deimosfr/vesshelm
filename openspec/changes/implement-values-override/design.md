# Design: Values Selection Implementation

## Architecture
- **Input**: `Chart` struct with `values: Option<Vec<Value>>` and `values_files: Option<Vec<String>>`.
- **Transformation**:
  - `values_files`: Directly map to `-f <path>`.
  - `values`:
    - Since `helm --set` has limitations with complex structures and types, we will use a temporary file strategy.
    - Create a `tempfile::NamedTempFile`.
    - Serialize the `values` vector (as a YAML sequence or merged map?) into this file.
    - Helm can accept multiple values files.
    - *Decision*: Attempt to merge `Vec<Value>` into a single YAML document or list? Helm `values.yaml` is usually a map. The config `values` is a `Vec<Value>`. If the user provides `[{a: 1}, {b: 2}]`, we should probably write this as a YAML list? No, Helm expects a map at the top level for values.
    - *Correction*: If `Chart.values` is a List of Maps, we should probably merge them or just write them as a list if Helm accepts it (Helm doesn't, it expects keys).
    - If the user writes:
      ```yaml
      values:
        - key: value
      ```
      This means they want to set `key` to `value`.
      If we write this array to a file, it's a YAML Array. Helm `values.yaml` must be a Map.
      So we must merge these items into a single Map, or simply assume the user provided valid YAML structure for `values`.
      Wait, if `values` is `Vec<Value>`, the user forces it to be a list in `vesshelm.yaml`.
      If `vesshelm.yaml` schema enforces `Vec`, then we have `[ {k: v}, {k2: v2} ]`.
      We should likely **merge** specific entries or write them as separate documents?
      Actually, the user example shows `values: - key: value`.
      If we pass this to logic, we should probably iterate and treat each item as a partial values object.
      But `helm -f` expects a file. If the file contains an Array, Helm might reject it or not doing what user expects (setting keys).
      *Refinement*: We will verify what `helm -f` expects. It expects a map.
      So we should verify if `config.values` can be serialized as a map.
      BUT `config.rs` defines it as `Option<Vec<Value>>`.
      So we have a List. E.g. `[{"a":1}, {"b":2}]`.
      If we serialize this "as is", it is ` - a: 1\n - b: 2`. This is an Array.
      Helm will probably complain "Error: unmarshal: json: cannot unmarshal array into Go value of type map[string]interface {}".
      **Solution**: We should iterate over the `Vec<Value>`, and for each `Value` (assuming it's a Map), merge it into a single Map (or write multiple temp files? No, single merged is better, or just iterate and write each to a temp file? No, that's wasteful).
      **Better Solution**: Flatten the list. Iterate `Vec<Value>`, ensure each is a Mapping. Merge them into a single `serde_yaml::Mapping`. Serialize that Mapping to the temp file.
      If an entry is NOT a mapping, log a warning/error.

## CLI Construction
- Construct args: `helm upgrade ... -f existing.yaml -f temp_values.yaml ...`
- The `temp_values.yaml` path must be passed.
- `NamedTempFile` persists until it goes out of scope. We must keep the handle alive until `execute_helm_command` finishes.
