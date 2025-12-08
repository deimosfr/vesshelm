# Proposal: Brighten CLI Colors

## Why?
The user requested all colors to be updated to "bright mode". Current CLI output uses a mix of standard and bright colors, leading to inconsistent visual weight and potential visibility issues on certain terminal backgrounds. Standardizing on bright colors will improve readability and vibrancy.

## What?
- Update all distinct color usages in the codebase to use the "bright" variant.
- Specifically, usages of `console::style(...).color()` should be updated to `console::style(...).color().bright()`.
- Affected colors likely include: green, red, yellow, blue, cyan, magenta.
- Ensure `src/util/ui.rs` helpers enforce this standard.

## Impact
- **User Experience**: providing a more vibrant and readable CLI output.
- **Codebase**: Widespread but mechanical changes across CLI command modules.
