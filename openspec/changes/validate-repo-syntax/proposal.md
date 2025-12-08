# Validate Repository Syntax

## Context

Users encountered issues with malformed `repositories` configuration in `vesshelm.yaml`, specifically indentation errors where `url` is not correctly nested under the list item.

## Problem

The current YAML parser or validation might not provide helpful error messages when `url` is incorrectly indented. The parser may treat misplaced keys as duplicates or invalid types at the wrong level, leading to confusing error messages.

## Solution

Update the specification to explicitly require validation of repository configuration structure and ensure clear error messages are reported for common YAML syntax errors like misplaced keys.
