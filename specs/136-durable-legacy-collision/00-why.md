# 00 — Why SPEC-136

## Trigger

[GitHub #377](https://github.com/raphaelmansuy/edgequake/issues/377): after SPEC-120 INSERT absorb, partner ingest still fails **retry-identically** on

```text
duplicate key value violates unique constraint
"idx_entity_embeddings_legacy_vector_id"
```

(and the relationship twin). Logs at 14:23 and 14:24 for the same document are the same 23505. Confirmed again on v0.25.0 via [#383](https://github.com/raphaelmansuy/edgequake/issues/383).

## Product WHY

```ascii
  User: reprocess the failed doc
       │
       ▼
  Winner FK already owns (workspace_id, legacy_vector_id)
  Loser FK is a different entities.id (display vs exact-normalized name)
  Loser already has a typed PK row with legacy_vector_id IS NULL
       │
       ▼
  stamp-once UPDATE SET lid WHERE fk = loser AND lid IS NULL
       │
       ▼
  SQLSTATE 23505  (UPDATE has no ON CONFLICT)
       │
       ▼
  Merger records GraphMerge → persister compensates
  Retry repeats the same UPDATE → fail forever
```

SPEC-120 closed **concurrent INSERT** of two new PKs (`ON CONFLICT DO NOTHING`). It did not close **stamp-once UPDATE** against a pre-existing NULL-lid PK.

## Success

1. Stamp UPDATE skips when the lid is already owned in the workspace, or absorbs 23505.
2. `upsert_batch` / merger `errors == 0` on that fixture; **retry** is also `errors == 0`.
3. Exactly one lid owner; loser does not steal the lid.
4. Unique index stays (LAW-120-2). HTTP dual-doc soak is **not** claimed.
