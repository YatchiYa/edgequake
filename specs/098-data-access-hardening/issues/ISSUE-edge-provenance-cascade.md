# ISSUE — Edge provenance SSOT / multigraph cascade (science_one)

| Field | Value |
|-------|-------|
| ID | F-098-21 |
| Law | LAW-098-13 |
| Symptom | F (`00-why.md`) |
| Repro | Delete `science_one.extracted.md` → **Delete failed**: `Post-proof failed: 0 nodes and 7 edges still reference document sources` |

## Root cause

1. `collect_source_references` treated edge topology `source_id` (`workspace::ENTITY`) as document provenance → exclusive edges never deleted; rebuild poisoned `source_ids` / `source_chunk_ids`.
2. Discovery/cascade collapsed on `(src, tgt)` while native arbiter is `(src, tgt, rel_type)` → multigraph sisters skipped.
3. `apply_rebuild_to_properties` left singular `source_chunk_id` / `source_document_id` uncleared; GIN discovery only probes `source_ids`.
4. **Accent drift (live science_one):** delete compared Rust-normalized labels to `eq_rel_type` with ASCII upper → `REPRéSENTE` ≠ `REPRÉSENTE` → **0 rows deleted**, post-proof still saw 7 singular-citation edges.

## Fix

- Provenance collector: arrays + singulars; reject `::` topology; legacy node pipe-join only when provenance-shaped.
- Rebuild clears/rewrites singular citation fields.
- Discovery + cascade identity `(src, tgt, rel)`; exclusive delete passes **raw** `properties.relation_type`.
- Bounded singular orphan discovery for poisoned arrays.
- **Delete SSOT = trigger formula** in SQL: `row.eq_rel_type = UPPER(COALESCE(NULLIF(TRIM(pairs.rel_type), ''), 'RELATED_TO'))` via `sql_eq_rel_type_arbiter_expr` — not Rust/Postgres dual heuristics.
- E2E: `e2e_spec098_accent_rel_delete_arbiter` (byte-level prop≠eq drift) + `e2e_spec098_edge_provenance_cascade`.

## Acceptance

- Retry Delete on science_one (and similar `delete_failed` post-proof docs) completes; AGE has no singular/array citation for that document id.
- CI gate `SPEC-098 edge provenance SSOT (LAW-098-13)` green.
