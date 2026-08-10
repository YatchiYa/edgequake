# ISSUE — Relation allowlist API

**Findings:** F-114-01, F-114-06, F-114-10  
**Wave:** W1  
**Laws:** LAW-114-1, LAW-114-2, LAW-114-4

## Goal

Persist `relation_types`, `relation_types_strict`, and optional `kg_schema_preset` on workspace metadata via existing create/update endpoints.

## Work

1. DRY: `normalize_type_list` (alias `normalize_entity_types`).  
2. Apply helpers for relation types / strict / preset.  
3. Extend core + API Create/Update/Response DTOs.  
4. `workspace_to_response` surfaces fields.  
5. Unit + API e2e round-trip.

## Acceptance

- Empty list clears key (free-form).  
- Max 50 cap.  
- Strict sparse encoding mirrors entity.  
- OpenAPI / clients updated if required.
