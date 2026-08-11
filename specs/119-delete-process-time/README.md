# SPEC-119 — Delete/Reprocess Timeout on Missing Singular Edge Indexes

> **Mission:** Make document delete/reprocess complete within the 2s source-discovery budget by indexing singular edge citation props and aligning filter expressions so Postgres can use Index Cond instead of Seq Scan.  
> **Trigger:** [GitHub #375](https://github.com/raphaelmansuy/edgequake/issues/375) — `Source-prefix singular edge query failed: … statement timeout`.

## Short verdict

| Layer | Finding |
|-------|---------|
| Symptom | Delete/reprocess cancels on SPEC-098 Symptom F singular-edge probe |
| Budget | `SOURCE_DISCOVERY_STATEMENT_TIMEOUT_MS = 2000` |
| Gap 1 | No btree on `"EDGE"` for `source_chunk_id` / `source_document_id` |
| Gap 2 | Singular SQL uses `::jsonb` cast → defeats btree even if added (same class as #362) |
| Wrong fix | Parent `_ag_label_edge` indexes (M036 style) — live path is child `"EDGE"` |

```ascii
  DELETE / reprocess
       │
       ▼
  find_edges_by_source_prefixes
       ├─ modern source_ids @>     → GIN OK
       └─ singular chunk/doc id    → need btree + exact expression
              │
              ▼
         Index Cond / BitmapOr  (≤ 2s)
```

## Document map

```ascii
 00-why
   → 01-first-principles (LAW-119-1..7)
   → 02-cross-ref-matrix
   → 03-code-as-is
   → 04-target-architecture
   → 05-lenses/ (PO, fullstack, DB, UX, front, marketing)
   → 06-ux-ui-spec
   → 07-implementation-plan
   → 08-test-protocol
   → 09-acceptance
   → 10-reproduction
```

## Status board

| ID | Item | Status |
|----|------|--------|
| D1 | Doc pack | Done |
| R1 | Local EXPLAIN reproduction | Done |
| G1 | GitHub #375 investigation + fix + ops comments | Done |
| I1 | Align singular SQL (no `::jsonb` on `->>`) | Done |
| I2 | `ensure_indexes` singular btrees | Done |
| I3 | sqlx marker migration 145 | Done |
| I4 | DRY `graph_cleanup_timeout` SSOT + delete UX | Done |
| I5 | `retract_document_indexes_checked` for reprocess | Done |
| T1 | EXPLAIN chunk_id + document_id + OR BitmapOr | Done |
| T2 | Wall e2e (200 edges) + live OR Index Cond | Done |
| T3 | Retract clears singular-only edges (memory) | Done |
| A1 | Acceptance (honest limits listed) | Done |

## Related

- [Issue #375](https://github.com/raphaelmansuy/edgequake/issues/375)
- [Issue #331](https://github.com/raphaelmansuy/edgequake/issues/331) (vertex GIN precedent — child `"Node"`)
- [Issue #362](https://github.com/raphaelmansuy/edgequake/issues/362) (cast defeating index)
- SPEC-098 Symptom F (`specs/098-data-access-hardening/`)
- SPEC-071 edge source-prefix GIN
- SPEC-091 RM3 citation indexes / M137

## Non-goals (v1)

- Raising discovery timeout as the primary fix
- Recreating parent `_ag_label_edge` property indexes
- Removing Symptom F singular discovery
- Full delete batching / async pagination redesign (follow-up if Index Cond insufficient at extreme scale)
