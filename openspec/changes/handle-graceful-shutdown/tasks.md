# Tasks

- [x] Add `tokio::signal` dependency in `Cargo.toml` if not already implied by `tokio` (tokio full feature covers it)
- [x] Implement a `SignalHandler` or modify `execute_helm_command` to listen for Ctrl+C
- [x] Ensure that when a signal is received:
    - A message "Received interrupt, waiting for helm..." is printed.
    - The child process (Helm) receives a SIGTERM.
    - `vesshelm` waits for the child process to exit.
- [x] Verify that `vesshelm` exits with a non-zero status code after interruption.
