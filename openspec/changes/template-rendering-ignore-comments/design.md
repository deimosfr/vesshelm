# Design: Ignore commented lines during template rendering

## Goal
Modify `render_template` to pre-process template content by "hiding" YAML-style comment lines from the `minijinja` engine.

## Context
Files being rendered are often YAML files where some lines are commented out using `#`. These lines might contain `{{ ... }}` placeholders that are not part of the active configuration and thus not present in the rendering context. With `minijinja`'s `Strict` behavior, this causes errors that frustrate users.

## Goals / Non-Goals

**Goals:**
- Skip Jinja variable interpolation for lines that are entirely comments.
- Preserve line numbers in `minijinja` error reports to ensure accurate error locations.
- Ensure resulting rendered string is still valid YAML (though possibly with many empty lines instead of comments).

**Non-Goals:**
- Handling "partial" comments (e.g., `key: value # more {{ jinja }}`), as parsing this correctly without a full YAML parser is complex and could break legitimate template usage.
- Removing comments *permanently* if the user wanted them in the output (though since we parse the result as YAML immediately in `load_variables`, they would be lost anyway).

## Decisions
1. **Pre-processing**: Instead of using regex or a complex parser, I'll use simple line-by-line processing.
2. **Replacement**: Lines matching the comment pattern `^\s*#.*` will be replaced by an empty line in the content string before `minijinja` sees it.
3. **Internal Helper**: I'll either modify `render_template` directly or introduce a small utility function for this pre-processing step.

## Risks / Trade-offs
- **Line Endings**: On Windows, line endings might be `\r\n`. Using `lines()` and `join("\n")` might normalize line endings, which is generally fine for YAML but should be noted.
- **Yaml edge cases**: Technically a YAML comment can start anywhere, not just at the beginning of a line. However, most common "disabled" blocks in Helm/YAML values are whole lines.
- **Multiline strings**: If someone has a YAML multiline string that happens to contain lines starting with `#`, they'll be stripped incorrectly. Example:
  ```yaml
  script: |
    # This is part of the string, not a YAML comment!
    ls -l
  ```
  This is a risk. However, for "values files" used as variables context, this is less frequent than commented-out configuration. I'll stick to simple line-start stripping for now as per "ignore commented lines".
