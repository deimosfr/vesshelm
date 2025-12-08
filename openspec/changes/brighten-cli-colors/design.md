# Design: Bright CLI Colors

## Goals
- **Consistency**: All colored text should use the high-intensity (bright) ANSI equivalents.
- **Vibrancy**: The CLI should feel "modern" and "pop" more.

## Implementation Details

The `console` crate provides a `.bright()` method on `Style` objects.

### Pattern
Existing code:
```rust
style("Success").green()
```

New code:
```rust
style("Success").green().bright()
```

### Centralization
Where possible, we should leverage `src/util/ui.rs` tocentralize styling. However, a full refactor to move all UI printing to `ui.rs` is out of scope. We will apply the `.bright()` modifier in-place or via existing helpers.

### Color Mapping
- `green` -> `green().bright()`
- `red` -> `red().bright()`
- `yellow` -> `yellow().bright()`
- `blue` -> `blue().bright()`
- `cyan` -> `cyan().bright()`
- `magenta` -> `magenta().bright()`
- `white` -> `white().bright()` (if used)
- `black` -> `black().bright()` (usually dark gray, risky for visibility, check context)

## Verification
- Visual inspection of `cargo run` output for key commands (`deploy`, `sync`, `validate`).
- No functional regressions.
