# Allow Direct Path in Chart Destination
type: change
status: proposed
owner: user
title: Allow Direct Path in Chart Destination

## Problem
Currently, the `dest` field in a chart configuration is treated strictly as a key to look up a named destination in `destinations`. This forces users to define a destination for every path they want to use, which is verbose for one-off paths (like plugins).
Additionally, the field is named `destination_override` in the struct but users expect `dest` in YAML, which causes the field to be ignored/swallowed by the parser if not correctly aliased, leading to unexpected behavior (syncing to default).

## Solution
1.  Update `Chart` configuration to properly map `dest` YAML field.
2.  Allow `dest` to be interpreted as a direct file system path if it does not match any named destination.
