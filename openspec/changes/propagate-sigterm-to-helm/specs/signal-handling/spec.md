# Signal Handling Requirements

## MODIFIED Requirements

### Requirement: Signal Propagation Logic
vesshelm MUST properly propagate termination signals to its child processes and ensure they exit cleanly.

#### Scenario: User interrupts deployment with Ctrl+C
When the user presses Ctrl+C during a locally running deployment:
1. `vesshelm` MUST catch the SIGINT signal.
2. `vesshelm` MUST forward the SIGINT signal to the active Helm child process.
3. `vesshelm` MUST WAIT for the Helm process to exit before exiting itself.
4. If Helm exits gracefully, `vesshelm` SHOULD report the interruption and exit with a non-zero code.

#### Scenario: CI/CD sends SIGTERM
When `vesshelm` receives a SIGTERM signal (e.g., from Kubernetes, Jenkins, or `kill`):
1. `vesshelm` MUST catch the SIGTERM signal.
2. `vesshelm` MUST forward the SIGTERM signal to the active Helm child process.
3. `vesshelm` MUST WAIT for the Helm process to exit.

#### Scenario: Helm hangs during cancellation
If the Helm process does not exit within a configurable timeout (default 30 seconds) after receiving the signal:
1. `vesshelm` MUST send a SIGKILL (Force Kill) to the Helm process.
2. `vesshelm` MUST exit with an error indicating the process was force-killed.
