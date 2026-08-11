# 07 — Implementation Plan

## Principles

- **DRY:** singular extract expression matches `idx_edge_source_id` style; timeout detection reused
- **SOLID:** DDL in `ensure_indexes`; query in `scan_ops`; cascade unchanged
- **First principles:** index + exact expression; do not raise timeout as the fix
- **Test first:** EXPLAIN Index Cond before claiming scale

## Phase A — Query expression alignment

1. In `scan_ops.rs` singular SQL, replace:
   - `({props})::jsonb->>'source_chunk_id'`
   - `({props})::jsonb->>'source_document_id'`
   with:
   - `{props}->>'source_chunk_id'`
   - `{props}->>'source_document_id'`
2. Leave modern GIN `({props})::jsonb -> 'source_ids'` unchanged.

## Phase B — Indexes

1. Add to `graph_lifecycle::ensure_indexes` edge section:
   - `idx_edge_source_chunk_id`
   - `idx_edge_source_document_id`
2. Add `145_spec119_edge_singular_citation_indexes.sql` marker (mirror M137).

## Phase C — Tests

1. Contract: indexes exist after ensure_indexes / upsert.
2. EXPLAIN: singular equality uses Index Cond (not Seq Scan); not `_ag_label_edge`.
3. Wall budget: `find_edges_by_source_prefixes` with singular-only edges under 2s.
4. Keep existing Symptom F / cascade e2e green.

## Phase D — UX mapping (lightweight)

1. Where deletion failure reason is formatted, map statement-timeout / singular-edge timeout to product copy (reuse existing detector if practical).
2. Docs only if code path is too wide for a small change.

## Phase E — Docs / GitHub

1. Comment on #375 with findings + SPEC-119 link.
2. Update README status board + acceptance.

## Edge-case matrix

| ID | Case | Mitigation | Test |
|----|------|------------|------|
| EC-01 | Missing singular indexes | ensure_indexes + marker | contract |
| EC-02 | `::jsonb` cast defeats btree | rewrite singular filter | EXPLAIN Index Cond |
| EC-03 | OR chunk_id OR document_id | two btrees | EXPLAIN BitmapOr/Index |
| EC-04 | Singular-only poisoned edge | Symptom F probe | memory + PG cascade |
| EC-05 | Modern GIN unchanged | leave `::jsonb -> source_ids` | e2e_spec071 |
| EC-06 | Empty / large probes | early-return + LIMIT 5000 | unit |
| EC-07 | New workspace graph | ensure_indexes on init | contract after upsert |
| EC-08 | Parent leftover M036 | do not recreate parent | EXPLAIN child only |
| EC-09 | Delete under 2s | Index Cond wall budget | e2e wall |
| EC-10 | Reprocess same path | retract → same discovery | existing reprocess e2e |
| EC-11 | UI raw timeout | map/error copy | UX + optional API assert |
| EC-12 | Null singular props | btree skips nulls | EXPLAIN with real probe |

## Rollout

1. Land docs + code + tests.
2. On upgrade, `ensure_indexes` creates btrees (startup may take longer once on large EDGE — operational note).
3. Close #375 when acceptance green.
