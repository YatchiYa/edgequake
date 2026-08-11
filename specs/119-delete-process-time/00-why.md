# 00 — Why SPEC-119

## Trigger

Users delete or reprocess a document. On large graphs the job fails with:

```text
Deletion failed: Storage error: Database error:
  Source-prefix singular edge query failed:
  error returned from database: canceling statement due to statement timeout
```

Reported on v0.24.2 / v0.24.3 against ~220k+ edges — [GitHub #375](https://github.com/raphaelmansuy/edgequake/issues/375).

## Product WHY

```ascii
  User expects: Delete / Reprocess finishes
       │
       ▼
  System must prove: no edge still cites this document
       │
       ├─ modern citations: source_ids[]          (GIN indexed)
       └─ Symptom F leftovers: singular props    (NOT indexed)
              │
              ▼
         Seq Scan under 2s budget → TIMEOUT → trust broken
```

Without reliable delete/reprocess:

- Poisoned / orphan edges remain → RAG cites dead documents
- Operators raise `statement_timeout` manually → not viable for UI
- Graph size growth makes every workspace eventually hit the cliff

## Gaps

| Approach | Gap |
|----------|-----|
| M036 edge property indexes | Only `source_id` / `target_id` / `workspace_id` / `tenant_id` — not citation singulars |
| SPEC-091 `idx_edge_source_chunk_ids_gin` | Plural array GIN for `@>` — does not serve `->>'source_chunk_id'` |
| SPEC-098 Symptom F probe | Correct correctness fix; never cross-checked for index coverage |
| Issue #375 suggested DDL | Targets parent `_ag_label_edge` (obsolete post-M070); live queries hit child `"EDGE"` |
| Singular SQL `::jsonb` cast | Even a correctly named btree would be unused (expression mismatch — #362 class) |

## Success

1. Singular-edge discovery uses Index Cond (or BitmapOr of Index Scans) on child `"EDGE"`.
2. Delete/reprocess completes within `SOURCE_DISCOVERY_STATEMENT_TIMEOUT_MS` (2s) on large graphs.
3. CI proves plan shape + wall budget; cascade still finds singular-only edges.
4. Users do not see raw Postgres cancellation as the primary error copy (bounded product message when timeout still occurs).
