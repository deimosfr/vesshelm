# Example Refinement Specs

## MODIFIED Requirements

### Requirement: Automated Example Generation

The system MUST be able to generate example configurations and documentation via tests to ensure they remain valid. The examples MUST be named according to their functional purpose.

#### Scenario: Generate Basic Example (Refined)

The "basic" scenario (previously metrics-server) MUST result in a directory named `examples/simple-usage`.
It still uses metrics-server as the content but the folder and identity reflect "simple usage".

#### Scenario: Generate Complex Example (Refined)

The "complex" scenario (previously external-dns) MUST result in a directory named `examples/complex-overrides`.

#### Scenario: Generate Variable Precedence Example (Refined)

The `examples/variable-precedence` example MUST demonstrate overriding `image.pullPolicy` using a variable from `overrides/common.yaml`.
The `vesshelm.yaml` should use `helm_args_append` or specific value overrides to set `image.pullPolicy={{ common_policy }}` or similar.
