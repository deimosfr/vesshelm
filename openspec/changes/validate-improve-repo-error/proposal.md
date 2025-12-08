# [Proposal] Improve Validation Error for Missing Repositories

## Goal Description

Improve the error message when a chart references a repository that does not exist in the configuration. Currently, the message is generic ("Chart references a repository that does not exist"), making it difficult to identify the problematic chart and repository.

## Proposal

- Update the validation logic to capture the chart name and the missing repository name.
- Update the error formatting to display these details in the error message.

## Rationale

Explicitly naming the chart and the missing repository significantly reduces debugging time for users with large configurations.
