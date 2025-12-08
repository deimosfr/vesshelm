# Proposal: Refine Example Tests

## Background

The initial implementation of example tests named examples after the charts they used (e.g., `metrics-server`). The user requested that examples be named after their _goal_ or _purpose_ (e.g., `basic-usage`, `complex-values`) to be more descriptive. Additionally, the `variable-precedence` example used a poor override target.

## Goal

1.  Rename "metrics-server" example to "simple-usage" (or similar functional name).
2.  Rename "external-dns" example to "complex-overrides".
3.  Update `variable-precedence` to override `image.pullPolicy` using a variable, instead of an arbitrary argument.

## Solution

- Refactor `tests/examples_test.rs` to change the directory names and `ExampleGenerator` keys.
- Update the `vesshelm.yaml` content in `test_variable_precedence_example` to target `image.pullPolicy` with the interpolated variable.

## Scope

- `tests/examples_test.rs`
- Generated `examples/` directories (will need to clean up old ones or just let `clean()` method handle the new ones, and manually delete old ones if desired, though the test harness only cleans its target).
