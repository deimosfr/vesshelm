# Design: Prettify Sync Output

## UI Enhancements
1.  **Colors & Style**: Use the `console` crate (or `colored`) to add bold text and colors (Green for success, Red for failure, Cyan for info).
2.  **Icons/Emojis**:
    -   Helm Repo: 📦
    -   Git Repo: 🐙
    -   OCI Repo: 🐳
    -   Success: ✓
    -   Failure: ✗
3.  **Progress Feedback**:
    -   Maintain `indicatif` spinner but style it better.
    -   Print a persistent line *after* each chart finishes (cleaning up the spinner).
4.  **Summary**:
    -   At the end, print a summary: "Synced: X, Failed: Y, Skipped: Z".

## Dependencies
- Add `console` crate for terminal formatting.
- `indicatif` matches well with `console`.

## Logic Changes
- In `src/cli/commands/sync.rs`, track counts of success/failure/skip.
- Instead of just `println!`, use formatted output.
