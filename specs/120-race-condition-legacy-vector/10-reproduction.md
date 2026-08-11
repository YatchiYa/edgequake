# 10 — Reproduction

## Environment

- Product pin: **v0.24.3** / HEAD
- Postgres: local `edgequake-postgres` with migrations 143/144 applied
- Indexes confirmed:

```text
idx_entity_embeddings_legacy_vector_id
  UNIQUE (workspace_id, legacy_vector_id) WHERE legacy_vector_id IS NOT NULL
```

## Pre-fix SQL proof (2026-08-11)

Forces two distinct `entity_id`s with the same `(workspace_id, legacy_vector_id)` using the **current** `ON CONFLICT (model_id, entity_id) DO UPDATE` shape:

```sql
BEGIN;
DO $$
DECLARE
  t uuid := gen_random_uuid();
  w uuid := gen_random_uuid();
  e1 uuid := gen_random_uuid();
  e2 uuid := gen_random_uuid();
  mid uuid;
  emb halfvec(3) := '[0.1,0.2,0.3]'::halfvec;
BEGIN
  INSERT INTO tenants (tenant_id, name, slug) VALUES (t, 'repro374-'||t, 'r374-'||t);
  INSERT INTO workspaces (workspace_id, tenant_id, name, slug) VALUES (w, t, 'w', 'w'||w);
  INSERT INTO entities (id, name, entity_type, description, tenant_id, workspace_id, sync_status)
    VALUES (e1, 'JOHN_SMITH', 'PERSON', 'a', t, w, 'synced'),
           (e2, 'John Smith', 'PERSON', 'b', t, w, 'synced');
  INSERT INTO embedding_models (name, dimensions)
    VALUES ('repro374', 3)
    ON CONFLICT (name, dimensions) DO UPDATE SET name = EXCLUDED.name
    RETURNING id INTO mid;

  INSERT INTO entity_embeddings
    (model_id, entity_id, workspace_id, embedding, dimensions, legacy_vector_id)
  VALUES (mid, e1, w, emb, 3, 'entity:JOHN_SMITH');

  BEGIN
    INSERT INTO entity_embeddings
      (model_id, entity_id, workspace_id, embedding, dimensions, legacy_vector_id)
    VALUES (mid, e2, w, emb, 3, 'entity:JOHN_SMITH')
    ON CONFLICT (model_id, entity_id) DO UPDATE
      SET legacy_vector_id = COALESCE(
            entity_embeddings.legacy_vector_id, EXCLUDED.legacy_vector_id);
    RAISE NOTICE 'UNEXPECTED: second insert succeeded';
  EXCEPTION WHEN unique_violation THEN
    RAISE NOTICE 'REPRO OK: % — constraint=%', SQLERRM, SQLSTATE;
  END;
END $$;
ROLLBACK;
```

### Observed

```text
NOTICE:  REPRO OK: duplicate key value violates unique constraint
         "idx_entity_embeddings_legacy_vector_id" — constraint=23505
```

## Relevance verdict

| Question | Answer |
|----------|--------|
| Still on v0.24.3? | **Yes** — mirror upsert unchanged vs issue |
| Issue entity-create claim exact? | **Nuanced** — exact-name sink is ON CONFLICT-safe; dual-FK via alias/resolve still hits legacy unique |
| Migration 144 sufficient? | **No** — cross-WS only |

## Post-fix verification

Absorb semantics (`INSERT … ON CONFLICT DO NOTHING`) on the same dual-FK setup:

```text
NOTICE:  POST-FIX OK: owners=1 (expect 1)
```

Automated:

```bash
export DATABASE_URL=postgresql://edgequake:edgequake_secret@localhost:5432/edgequake
export EDGEQUAKE_REQUIRE_POSTGRES_TESTS=1
cargo test -p edgequake-storage --features postgres \
  --test contract_spec120_legacy_vector_id_race \
  --test e2e_spec120_concurrent_mirror_same_entity
```

Result (2026-08-11): **6 + 2 passed**.
