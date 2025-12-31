# Proposal: Add Command

**change-id**: `add-command`

## Summary
Implement a `vesshelm add` command to simplify adding new charts to the configuration. The command will use an interactive wizard to guide the user through adding a chart from an Artifact Hub URL, automatically detecting repository and chart details.

## Motivation
Currently, users must manually edit `vesshelm.yaml` to add charts and repositories. This is error-prone and requires looking up details like repository URLs and chart versions. The `add` command will automate this process, reducing friction and ensuring consistency.

## Solution
The `add` command will:
1.  Prompt for an Artifact Hub URL.
2.  Fetch chart details (repository URL, chart name, version) from Artifact Hub.
3.  Check if the repository exists in `vesshelm.yaml` and add it if missing.
4.  Prompt for configuration details (alias, namespace).
5.  Generate the chart configuration block.
6.  Preview the changes and urge the user to confirm.
7.  Update `vesshelm.yaml`.
