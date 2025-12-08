# Change: Prettify Deploy Output

## Why
The current `deploy` command output uses structured logging (tracing) which includes timestamps, log levels (INFO), and module paths. This is too verbose and technical for a CLI user interface. Users prefer a cleaner, more "fancy" output that focuses on what is happening (Deploying X, Skipping Y) without the noise.

## What Changes
- Replace `tracing::info!` calls in `src/cli/commands/deploy.rs` with `println!` using `colored` or `console` crates for styling.
- Remove timestamps and log levels.
- Use emojis or clear status indicators if appropriate (aligned with other commands).

## Impact
- Affected specs: `chart-deployment`
- Affected code: `src/cli/commands/deploy.rs`
