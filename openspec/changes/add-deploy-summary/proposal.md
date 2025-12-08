# Add Deploy Command Summary

## Context
The `sync` command provides a useful summary of operations (Synced, Failed, Skipped). The `deploy` command currently ends with a single success message.

## Problem
Users lack a concise overview of the deployment results, such as how many charts were successfully deployed, how many failed, and how many were skipped.

## Solution
Update the `deploy` command to track and display a summary at the end of execution, matching the style of the `sync` command summary.

## Impact
Improved user experience and consistency between commands.
