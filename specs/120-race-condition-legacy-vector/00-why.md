# 00 — Why SPEC-120

## Trigger

Concurrent document/email ingestion into the same workspace can fail with:

```text
Knowledge graph persist failed: Graph error:
  1 knowledge-graph merge error(s) during persist:
  Storage error: Database error:
  duplicate key value violates unique constraint
  "idx_entity_embeddings_legacy_vector_id"
```

(and the relationship equivalent on `idx_relationship_embeddings_legacy_vector_id`).

Reported after upgrade to v0.24.2+ (migrations 143/144) — [GitHub #374](https://github.com/raphaelmansuy/edgequake/issues/374). Confirmed still present on **v0.24.3 / HEAD**.

## Product WHY

```ascii
  User expects: concurrent ingest converges
       │
       ▼
  System stamps typed fleet rows with legacy_vector_id
  (bookkeeping for migration 131 / provenance)
       │
       ├─ PK (model_id, entity_id)     → ON CONFLICT DO UPDATE (handled)
       └─ UNIQUE (workspace_id, lid)   → unhandled → HARD FAIL
              │
              ▼
         Entire document merge fails
         Real extracted KG may compensate/rollback
         Operators retry / lower concurrency (workaround)
```

Without absorb:

- Provenance metadata outranks extracted content (wrong priority)
- Race becomes visible only after 0.24.2 unique index (silent duplicates before)
- Partner trust eroded: “upgrade broke concurrent email ingest”

## Gaps

| Approach | Gap |
|----------|-----|
| Migration 143 unique on `legacy_vector_id` | Made race loud; no absorb path |
| Migration 144 workspace scope | Fixed cross-WS Acc collisions only |
| `ON CONFLICT (model_id, entity_id)` | Does not arbiter legacy unique ([Postgres INSERT](https://www.postgresql.org/docs/current/sql-insert.html): one conflict target) |
| Issue claim “entity create not race-safe” | Overstated for exact-name sink (`entities_unique_name` + ON CONFLICT); dual-FK still reachable via alias / resolve |
| `fleet_provenance_stamp` 23505 → Failed | Migration job path; not live ingest absorb |
| Reduce concurrency workaround | Operational only; not a product fix |

## Success

1. Same-workspace dual writers with the same `legacy_vector_id` and different FKs both return `Ok` from mirror/upsert.
2. Exactly one typed row owns each `(workspace_id, legacy_vector_id)`.
3. Document merge does **not** surface GraphMerge for absorbable legacy collisions.
4. Multi-workspace same lid still allowed (144 invariant).
5. Stamp-once: non-null `legacy_vector_id` is not overwritten.
6. Contract + e2e prove concurrency (LAW-120-7).
