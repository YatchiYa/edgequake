# Lens 003 — Database Expert

## Indexes in play

```ascii
  entity_embeddings
    PK (model_id, entity_id)
    UNIQUE (workspace_id, legacy_vector_id) WHERE lid IS NOT NULL
         ↑ migration 144 — keep
```

Same for relationship/report.

## Postgres constraint

Per [INSERT … ON CONFLICT](https://www.postgresql.org/docs/current/sql-insert.html):

- `DO UPDATE` **requires** a conflict target → can cover only one arbiter.
- `DO NOTHING` **without** target handles all unique/exclusion violations.

## Chosen SQL shape

```sql
-- (1) stamp-once existing PK
UPDATE entity_embeddings AS ee
SET legacy_vector_id = COALESCE(ee.legacy_vector_id, NULLIF(t.lid, ''))
FROM unnest(...) AS t(e, w, v, d, lid)
WHERE ee.model_id = $1 AND ee.entity_id = t.e
  AND ee.legacy_vector_id IS NULL
  AND NULLIF(t.lid, '') IS NOT NULL;

-- (2) insert new / absorb collisions
INSERT INTO entity_embeddings (...)
SELECT ...
FROM unnest(...) AS t(...)
ON CONFLICT DO NOTHING;
```

## Invariants preserved

| Invariant | How |
|-----------|-----|
| At most one lid per WS | Unique index unchanged |
| Same lid across WS OK | 144 composite key |
| Stamp-once | UPDATE only when `legacy IS NULL` |
| No 23505 to app | Targetless DO NOTHING |

## Explainability

Losing writer leaves an entity/rel row without a typed embedding for that FK when lid already taken — acceptable P0; query resolve prefers the stamped owner. Alias cleanup = SPEC-083.
