# Configure Release Automation

## Summary
Configure GitHub Actions to ensure code quality on Pull Requests and automate releases on tag pushes.

## Problem Statement
Currently, there is no automated CI/CD pipeline to verify code quality or handle releases. This manual process is error-prone and inefficient.

## Solution Strategy
Implement GitHub Actions workflows to:
1.  Verify builds, tests, and coverage on PRs.
2.  Automate releases using GoReleaser on tag pushes, ensuring version consistency.
