# Capability: Enhanced Sync UX

## MODIFIED Requirements

### Requirement: Sync command output must be rich and informative
The `sync` command MUST use colors and icons to distinguish status and types.

#### Scenario: Syncing charts
- Given I have a `vesshelm.yaml` with mixed repo types
- When I run `vesshelm sync`
- Then I see a spinner for active operations
- And I see "✓ Synced <chart>" in green upon success
- And I see "📦" for Helm charts and "🐙" for Git charts

#### Scenario: Sync Summary
- Given I run a sync operation
- When it finishes
- Then I see a summary line stating total synced, failed, and skipped charts
