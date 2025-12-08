# Propagate SIGTERM/SIGINT to Helm

## Problem
Currently, when `vesshelm` is interrupted via Ctrl+C (SIGINT) or receives a termination signal (SIGTERM), it attempts to stop the running Helm process. However, legitimate deployment interruptions often leave Helm releases in a "pending-upgrade" or "upgrading" state, requiring manual intervention to unlock. This suggests that the signal is either not being sent, not being sent correctly, or `vesshelm` exits before Helm has a chance to clean up its state. Specifically, the user reports that `vesshelm` "gives the hand immediately" (exits) without ensuring Helm has gracefully terminated.

## Solution
We need to improve the signal handling logic in the `deploy` command (and potentially others spawning Helm) to:
1.  Correctly intercept system signals (SIGINT, SIGTERM).
2.  Propagate the *same* signal to the child Helm process (e.g., SIGINT -> SIGINT, SIGTERM -> SIGTERM), as Helm might handle them differently (SIGINT for cancellation vs SIGTERM for forced stop).
3.  **Crucially**, wait for the child process to exit after sending the signal, ensuring it has time to release locks or clean up.
4.  Implement a timeout mechanism to force-kill if Helm hangs during cleanup.

## Risks
- If Helm truly hangs, `vesshelm` might hang waiting for it. A timeout is necessary.
- Different behavior on Windows vs Unix (Windows uses Ctrl+C propagation differently).
