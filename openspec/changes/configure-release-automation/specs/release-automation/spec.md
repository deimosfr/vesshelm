# Spec: Release Automation

## ADDED Requirements

#### Requirement: PR Code Quality Checks
The system MUST run a set of quality checks on every Pull Request.

#### Scenario: PR Compilation and Tests
Given a Pull Request is opened or updated
Then the system runs `cargo build` ensuring no errors
And the system runs `cargo build` ensuring no warnings
And the system runs `cargo test` ensuring no errors and no warnings
And the system runs `cargo tarpaulin` ensuring coverage is at least 60%
And the system runs `cargo audit` (allowing failure)

#### Requirement: Automated Release
The system MUST automate the release process when a tag is pushed.

#### Scenario: Tag Push Release
Given a new git tag is pushed
Then the system verifies the application version matches the tag
And the system runs `goreleaser` to publish the release

#### Requirement: Main Branch Silence
The system MUST NOT trigger workflows on pushes to `main` that are not tags.

#### Scenario: Push to Main
Given a push to `main` (not a tag)
Then no CI/CD workflow is triggered
