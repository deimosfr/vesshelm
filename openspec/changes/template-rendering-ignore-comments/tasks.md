# tasks for template-rendering-ignore-comments

## 1. Implement Comment-Ignoring Logic

- [x] 1.1 **Create pre-processing function**: In `src/util/variables.rs`, implement a function that takes a template string and replaces lines matching `^\s*#.*` with empty lines.
- [x] 1.2 **Update `render_template`**: Call the pre-processing function before passing the content to `minijinja::Environment`.
- [x] 1.3 **Handle line endings**: Ensure the logic works across different OS line endings if possible, or at least doesn't break standard usage.

## 2. Verify and Test

- [x] 2.1 **Add Unit Test**: In `src/util/variables.rs` under `mod tests`, add a test case that includes a commented-out Jinja expression with an undefined variable to ensure it no longer fails.
- [x] 2.2 **Add Test for Line Numbers**: Verify that error messages for other parts of the template still report correct line numbers.
- [x] 2.3 **Regression check**: Run existing tests in `src/util/variables.rs` to ensure no regressions for normal interpolation.
