# Allow Inline Values Map and Interpolation

Status: **Draft**
Authors: **Me** (on behalf of User)

## Summary

Allow the `values` field in the Chart configuration to be specified as a direct Map (key-value pairs) in addition to the current Sequence (list of maps). Additionally, enable variable interpolation within these inline values.

## Motivation

Currently, the `values` config only accepts a list of maps (Sequence). Users attempting to provide a simple map of value overrides encounter a parsing error: `invalid type: map, expected a sequence`.

Example failing config:
```yaml
values:
  key: value
```

This change improves developer experience by supporting standard YAML map syntax for values and adds power by allowing dynamic values via interpolation.

## Proposed Changes

1.  **Configuration Schema**: Update `Chart` struct to accept `values` as either a `Sequence` (existing behavior) or a `Map` (new behavior).
2.  **Interpolation**: Apply Jinja2 variable interpolation to inline values, consistent with how `values_files` are handled.

## Alternatives Considered

- **Wrap in list**: Force users to always write `values: [{key: value}]`. This works now but is verbose and unintuitive for simple overrides.
- **Separate field**: Add `values_map` field. This complicates the API.
