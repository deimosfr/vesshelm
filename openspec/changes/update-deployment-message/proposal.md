# Update Deployment Message

## Summary
Change the final success message of the `deploy` command from "Deployment completed" to "Deployment ended".

## Motivation
User preference for the wording "Deployment ended" to signify the conclusion of the process.

## Proposed Solution
- Update the `println!` statement in `src/cli/commands/deploy.rs`.
