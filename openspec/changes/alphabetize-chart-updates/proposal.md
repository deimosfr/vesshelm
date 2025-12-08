# Alphabetize Chart Updates

## Why
Currently, `vesshelm check-updates` checks charts in the order they appear in `vesshelm.yaml`. As the number of charts grows, it becomes harder for users to predict the output order or find specific charts in the logs. Alphabetical sorting provides a consistent and predictable user experience.

## What Changes
- In `check_updates.rs`, before iterating over `config.charts`, sort the vector by `chart.name`.
- This ensures the output "checking [chart]..." appears in alphabetical order.
