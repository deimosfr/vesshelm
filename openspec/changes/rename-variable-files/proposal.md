# Proposal: Rename variable_files to variables_files

## Summary
Rename the configuration field `variable_files` to `variables_files` (plural) to align with `secrets_files` naming convention. Maintain backward compatibility by keeping `variable_files` as an alias.

## Motivation
Consistency in configuration naming improves developer experience. `secrets_files` uses the plural form, so `variable_files` should also uses the plural form `variables_files`.

## Goals
- Rename `Config.variable_files` to `Config.variables_files`.
- Add `#[serde(alias = "variable_files")]` to `variables_files`.
- Update all internal code references.

## Non-Goals
- Changing the behavior of variable loading.
