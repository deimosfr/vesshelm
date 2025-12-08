# Fix Icon Spacing

## Goal
Ensure all output messages with icons have a space between the icon and the text that follows.

## Requirements
1.  **Consistent Spacing**: Every printed message that starts with an icon (emoji) MUST have a single space character immediately following the icon.
2.  **Scope**: This applies to all CLI commands (`deploy`, `sync`, `uninstall`, `validate`, `init`).

## Non-Goals
-   Changing the icons themselves.
-   Adding icons to messages that don't have them.

## Implementation Details
-   Inspect all `println!`, `eprintln!`, and `tracker.println` calls.
-   Ensure format strings use `"{}{} ..."` -> `"{}` `{}"` isn't used, but rather `format!("{} {}", icon, text)` or similar.
-   Common pattern to fix: `format!("{}{}", icon, text)` -> `format!("{} {}", icon, text)`.
