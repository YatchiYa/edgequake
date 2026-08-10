# ISSUE — Pipeline relation enforce

**Findings:** F-114-02  
**Wave:** W2  
**Laws:** LAW-114-3, LAW-114-5

## Goal

When workspace `relation_types` is non-empty, inject prompt guidance/strict section and enforce relation labels on parse/gleaning.

## Work

1. Extend extraction schema with `relation_types` + `relation_strict`.  
2. `from_workspace_metadata` (empty ⇒ free-form).  
3. Prompt fragment in `json_prompts.rs`.  
4. `enforce_relation_type` (fallback `RELATED_TO` or first).  
5. Wire factory; unit tests for strict/permissive.

## Acceptance

- Absent/empty: behavior identical to pre-SPEC-114.  
- Strict unknown → remapped.  
- Permissive unknown → normalized pass-through.
