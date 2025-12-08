# Proposal: Fix Deploy Command DAG Order

## Context
The `deploy` command currently uses a topological sort to determine the order of deployment. However, users have reported that the DAG order is not respected.

## Problem
Potential issues:
1. Stability of topological sort for independent nodes.
2. Filtering logic removing dependencies when specific charts are selected.
3. Incorrect edge direction in DAG construction.

## Solution By
Ensuring that:
1. The DAG sort is stable or deterministic.
2. Dependencies are correctly modeled.
3. (Optional) Missing dependencies are warned about or automatically included? (TBD based on spec)

## Goals
- `deploy` command guarantees parent charts are processed before child charts.
