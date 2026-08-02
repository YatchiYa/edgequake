# ISSUE — API persist (W1)

| Meta | Value |
|------|-------|
| Findings | F-102-01, F-102-05 |
| Laws | LAW-102-2, LAW-102-3 |
| Wave | W1 |
| Status | done |

## Problem

No `entity_type_colors` on workspace create/update/response.

## Approach

Mirror `entity_types`: request fields → `apply_entity_type_colors_metadata` → metadata JSONB → `WorkspaceResponse`. Validate hex; max 50; empty clears.

## DoD

- [x] Postgres + in-memory paths  
- [x] `spec102_entity_type_colors_persist` + invalid hex  
