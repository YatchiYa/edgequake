# LENS — Database Expert (SPEC-096)

> **Laws**: L1, L3, L5 · **Findings**: F-352-08 · **Pattern**: `entity_types` in workspace `metadata` JSONB

## Storage decision

**No schema migration.** Persist `extraction_language` inside existing `workspaces.metadata` JSONB (and in-memory metadata map for non-PG paths), identical to:

- `entity_types`
- `entity_types_strict`
- `llm_model` / embedding overrides

```json
{
  "entity_types": ["PERSON", "ORGANIZATION"],
  "extraction_language": "Chinese"
}
```

Absence of the key means “inherit env / default” (LAW-L3).

## Why not a column?

| Criterion | JSONB metadata | Dedicated column |
|-----------|----------------|------------------|
| Migration cost | Zero | New migration + backfill |
| Consistency with siblings | Matches entity_types / models | Divergent |
| Query need | No filter/index by language in v1 | Unnecessary |
| Evolution | Add keys freely | DDL churn |

If product later needs `WHERE metadata->>'extraction_language' = $1` at scale, add an expression index — **not** required for v1.

## Constraints

1. **Value size** — Allowlisted short strings only (EC-17); never store free-form essays.  
2. **Type** — JSON string, not object/array. Reject non-string at apply helper.  
3. **Clear** — Delete key on empty/`none` (do not store `null` unless existing helpers do for siblings — prefer **remove key**).  
4. **LAW-L5** — Language metadata change must not trigger AGE node UPDATE jobs.  
5. **Backup / restore** — Opaque JSONB; older binaries ignore unknown keys (forward compatible). Newer binaries reading old rows: missing key → resolve default.

## Indexes

None for v1. `workspace_id` PK already scopes reads.

## Multi-tenant isolation

Language is workspace-scoped metadata; no cross-workspace leakage beyond existing workspace ACL. No new RLS policies.

## In-memory / test adapters

`workspace_service/in_memory.rs` must apply the same metadata helper so unit tests without Postgres still round-trip.

## Verification

| Check | Method |
|-------|--------|
| Persist | `spec096_workspace_metadata_roundtrip` |
| No DDL | Roadmap DoD — `git diff` migrations empty for this feature |
| Compat | Old workspace rows without key still ingest (English/env) |

## Anti-patterns

- Storing language on every document row.  
- Storing language on AGE node properties as required field.  
- Enum PG type for languages (lockstep with Rust allowlist is harder than app validation).

## Laws

L1 (explicit stored contract), L3 (absence = inherit), L5 (metadata-only write).
