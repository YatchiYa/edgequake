# 01 — First Principles

## Axioms

1. Delete/reprocess is correct only if discovery finds **every** edge that cites the document.
2. Citation may live in plural arrays (`source_ids`, `source_chunk_ids`) **or** singular leftovers (`source_chunk_id`, `source_document_id`) — SPEC-098 Symptom F.
3. Interactive discovery is bounded by a hard statement timeout; unbounded Seq Scan is a product failure.
4. Postgres expression indexes match only when the **filter expression equals the index expression** ([Crunchy Data — Indexing JSONB](https://www.crunchydata.com/blog/indexing-jsonb-in-postgres)).
5. AGE live labels are child tables `"Node"` / `"EDGE"`; parent `_ag_label_*` indexes are not the serving path (M070 / ensure_indexes).
6. Indexes and query shape must evolve together (DRY SSOT for the property extract expression).

## Laws

| Law | Statement |
|-----|-----------|
| **LAW-119-1** | Symptom F singular discovery is part of delete/reprocess correctness, not optional |
| **LAW-119-2** | Index expressions must byte-match the filter (no `json` vs `jsonb` cast drift on `->>`) |
| **LAW-119-3** | Indexes live on child `"EDGE"` via `ensure_indexes` (+ sqlx marker), not parent `_ag_label_edge` |
| **LAW-119-4** | Discovery must complete within `SOURCE_DISCOVERY_STATEMENT_TIMEOUT_MS` as graph size grows |
| **LAW-119-5** | Product surfaces bounded/actionable failure; raw `statement_timeout` is not UX |
| **LAW-119-6** | One SSOT for singular citation property expressions (same pattern as `idx_edge_source_id`) |
| **LAW-119-7** | Contract + PG e2e prove Index Cond and delete/reprocess under the 2s budget |

## Causal diagram (Five WHYs)

```ascii
  WHY timeout?
    → singular probe Seq Scans all edges under 2s
  WHY Seq Scan?
    → no btree on source_chunk_id / source_document_id
       AND filter uses ::jsonb cast that would defeat btree
  WHY is the probe required?
    → Symptom F: poisoned source_ids leave only singular citation
  WHY not covered by existing GIN?
    → GIN indexes plural arrays for @>, not singular text equality
  WHY suggested parent indexes wrong?
    → M070 dropped parent serving indexes; queries read "EDGE"
```

## Normative expression (serving path)

```ascii
  Filter (must match index):
    ag_catalog.agtype_to_json(properties)->>'source_chunk_id'
    ag_catalog.agtype_to_json(properties)->>'source_document_id'

  FORBIDDEN on singular btree path:
    (ag_catalog.agtype_to_json(properties))::jsonb->>'…'
      └─ forces Seq Scan even when btree exists (proven vs idx_edge_source_id)
```
