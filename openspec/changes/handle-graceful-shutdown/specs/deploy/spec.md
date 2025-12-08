# Deploy Capability

## ADDED Requirements

### Graceful Shutdown
The system must handle interruption signals safely to prevent deployment locks.

#### Scenario: Vesshelm waits for Helm on interrupt
Given a deployment is in progress
And a `helm` child process is running
When the process receives a SIGINT (Ctrl+C) or SIGTERM
Then the system displays a "Stopping... please wait" message
And the system sends a termination signal to the `helm` process
And the system waits for the `helm` process to exit before terminating
And the process exits with a non-zero status code
