# Handle Graceful Shutdown

## Context
When a user presses `Ctrl+C` (SIGINT) or the process receives SIGTERM during a `vesshelm deploy`, the `vesshelm` process currently terminates immediately. If a `helm` command (like `helm upgrade`) is running as a child process, it might be abruptly killed or left orphaned, potentially leaving the Helm release in a locked or inconsistent state.

## Problem
Immediate termination prevents `helm` from cleaning up resources or finishing critical sections, leading to "release currently locked" errors on subsequent runs or partial deployments.

## Solution
Implement a signal handler (for SIGINT and SIGTERM) that:
1.  Intercepts the shutdown signal.
2.  Prints a user-friendly message acknowledging the signal and stating that `vesshelm` is waiting for the active Helm command to finish.
3.  Forwards the signal (SIGTERM) to the active `helm` child process if one exists, to request it to stop.
4.  Waits for the child process to exit gracefully before terminating `vesshelm`.

## Impact
- **Reliability**: Reduces the chance of stuck Helm locks.
- **UX**: Provides feedback that the exit is being handled safely ("Stopping... please wait").
