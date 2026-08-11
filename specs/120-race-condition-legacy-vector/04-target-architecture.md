# 04 — Target Architecture

## Serving path (P0)

```ascii
  mirror_legacy_batch  (unchanged resolve / row build)
       │
       ▼
  PgFleetEmbeddingIndex::upsert_batch
       │
       ▼
  fleet_legacy_absorb::upsert_with_legacy_absorb   [DRY SSOT]
       │  table/fk from EmbeddingFamily metadata
       ├─ (1) UPDATE by PK — stamp-once when legacy IS NULL
       ├─ (2) INSERT … ON CONFLICT DO NOTHING (targetless)
       └─ (3) count absorbed lid misses → never Err on legacy unique
```

## Resolve hygiene (P1)

```ascii
  load_entity_name_index*_pool
       │
       ▼
  SELECT id, name … ORDER BY created_at ASC, id ASC
       │
       ▼
  EntityNameIndex::from_rows — all keys use or_insert
    → oldest row wins for exact, normalized, and ws-suffix aliases
```

Full alias merge / spine cleanup remains SPEC-083.

## Observability

| Signal | Meaning |
|--------|---------|
| `MirrorLegacyReport.absorbed_legacy_collisions` | Loser lid writes skipped |
| `tracing::warn!` sample | Operator visibility without failing ingest |
| No GraphMerge | LAW-120-1 / LAW-120-6 |

## Out of scope (target v1)

- Merging alias `entities` rows into one spine
- Changing migration stamp job fail-closed dual-legacy policy
- Frontend changes
