# Design: Interactive CLI Progress UI

## Problem
Standard `println!` calls interfere with persistent progress bars (like those from `indicatif`), causing "tearing" or "glitches" where the bar is duplicated or broken by log output.

## Solution
We will introduce a `ProgressTracker` abstraction that wraps `indicatif::ProgressBar`.

### 1. Global Progress Bar
- **Position**: Bottom of the screen.
- **Style**:
  - Template: `{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({percent}%) {msg}`
  - Characters: Smooth block characters for the bar.
- **Behavior**:
  - Initialized with total count of charts.
  - incremented after each chart operation.
  - cleared/finished with a summary message upon completion.

### 2. Log Output Strategy
- **Rule**: All output during the operation MUST go through the progress bar handle.
- `indicatif` provides `pb.println(...)` which correctly prints *above* the persistent bar.
- We will replace `println!` macros in `deploy.rs` and `sync.rs` with calls to this tracker.
- For `helm` command output (child processes):
  - If we capture output, we print it via `tracker.info(...)`.
  - If we stream output (e.g. `inherit`), we might need `ProgressBar::suspend` blocks, but that causes flickering.
  - Better approach: Capture output and print line-by-line via `pb.println`, OR use `MultiProgress` but that's complex for just logs.
  - **Decision**: For `deploy`, we often want to see Helm logs. `vesshelm` currently captures some output but `deploy.rs` uses `execute_helm_command` that uses `status()` (inheriting stdio) or `output()`?
  - `execute_helm_command` uses `Command::new("helm").status()` which INHERITS stdin/stdout/stderr by default!
  - **Challenge**: Inherited stdout will collide with the progress bar.
  - **Fix**: We must pipe stdout/stderr and read it line-by-line, printing each line via `pb.println()`.

### 3. Architecture
```rust
struct ProgressTracker {
    pb: ProgressBar,
}

impl ProgressTracker {
    fn new(total: u64, message: &str) -> Self { ... }
    fn inc(&self) { self.pb.inc(1); }
    fn println(&self, msg: &str) { self.pb.println(msg); }
    fn finish(&self, msg: &str) { self.pb.finish_with_message(msg); }
}
```
