# Spec: dependency-management

## ADDED Requirements

### Requirement: Define Dependencies
The system MUST allow users to define dependencies between charts.

#### Scenario: Dependency Definition
- **WHEN** User defines `depends` list in optional chart config.
- **THEN** Deployment order respects these dependencies (dependent charts deploy after their prerequisites).

### Requirement: Cycle Detection
The system MUST detect and reject circular dependencies.

#### Scenario: Circular Dependency Loop
- **WHEN** Configuration contains a cycle (A depends on B, B depends on A).
- **THEN** System returns a fatal error identifying the cycle.

### Requirement: Dependency Validation
The system MUST validate that declared dependencies exist.

#### Scenario: Missing Dependency
- **WHEN** Chart A depends on "missing-chart".
- **THEN** System returns an error "Dependency 'missing-chart' not found".
