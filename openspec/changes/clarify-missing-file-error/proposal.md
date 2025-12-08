# Clarify Missing File Error

## Goal
Make configuration validation errors for missing variable and secret files explicit by including the missing file path in the error message.

## Context
Currently, when a variable file is missing, the error message is generic: `Error: Field '__all__' failed validation: variable_file_not_found`. It is difficult to identify which file triggered the error especially when multiple files are configured.

## Solution
Update the error formatting to include the file path that caused the validation failure.
