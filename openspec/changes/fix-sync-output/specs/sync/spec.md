## ADDED Requirements

### Requirement: Alphabetical Sync Execution
The `sync` command MUST process charts in alphabetical order by name.

#### Scenario: Charts are processed alphabetically
Given a configuration with charts `Zebra`, `Alpha`, `Beta`
When the sync command is executed
Then `Alpha` MUST be processed before `Beta`
And `Beta` MUST be processed before `Zebra`.
