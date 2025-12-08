# template-comments-ignoring Specification

## Purpose
Prevent Jinja rendering errors for commented-out YAML lines containing undefined variables.

## Requirements

### Requirement: Ignore YAML comments during rendering
Template rendering MUST ignore lines that consist solely of a YAML comment, ensuring placeholders within those comments are not evaluated by the Jinja engine.

#### Scenario: Comment with undefined variable
- **GIVEN** a YAML file with a comment: `# This is for {{ undefined_var }}`
- **AND** a valid key: `key: {{ some_var }}`
- **AND** `some_var` is defined in the context, but `undefined_var` is NOT.
- **WHEN** the template is rendered
- **THEN** it should succeed
- **AND** the rendered output should contain `key: <value>`
- **AND** the comment line should not cause any "undefined value" error.

#### Scenario: Mixed content on a line
- **GIVEN** a YAML line with a value and end-of-line comment: `key: {{ value }} # help: {{ info }}`
- **AND** `value` is defined, but `info` is NOT.
- **WHEN** the template is rendered
- **THEN** (Wait, should we handle this? "ignore commented lines" usually means full lines. Let's decide if we want to handle this or not based on common expectations. For now, since full lines are the most critical issue, we might start there.)
- **THEN** it might still fail if it's NOT a full-line comment. (Wait, let's keep it simple for now: "lines that consist only of comments").
