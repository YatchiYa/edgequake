# 01 — First Principles

## Axioms

1. Ingest correctness is measured by user content (entities, relationships, chunks), not by provenance bookkeeping alone.
2. `legacy_vector_id` is a **provenance stamp** bridging legacy `eq_*_vectors.id` → typed fleet rows (SPEC-111 / migration 131).
3. Uniqueness of `(workspace_id, legacy_vector_id)` is intentional: one logical legacy key → one typed stamp owner per workspace.
4. Postgres `ON CONFLICT` can target **one** arbiter per statement; `DO UPDATE` requires a target; targetless `DO NOTHING` absorbs **all** unique violations ([docs](https://www.postgresql.org/docs/current/sql-insert.html)).
5. Exact-name entity/relationship creation already uses UNIQUE + `ON CONFLICT` (LAW-120-4).
6. Concurrent writers are a supported product mode; lowering concurrency is not a fix.

## Laws

| Law | Statement |
|-----|-----------|
| **LAW-120-1** | Bookkeeping ≠ content — `legacy_vector_id` collision must never fail user-visible ingest |
| **LAW-120-2** | One logical key → one typed stamp owner per `(workspace_id, legacy_vector_id)` |
| **LAW-120-3** | Arbiter completeness — every unique index on the hot-path INSERT must be handled |
| **LAW-120-4** | Exact-name create is already race-safe — do not re-litigate sink UNIQUE; fix mirror + resolve hygiene |
| **LAW-120-5** | Alias/duplication is separate completeness debt (SPEC-083) — must not block P0 absorb |
| **LAW-120-6** | Absorbable 23505 must not trigger GraphMerge compensation against the winner |
| **LAW-120-7** | Prove with concurrency — contract + e2e same-workspace dual writers |

## Causal diagram (Five WHYs)

```ascii
  WHY document ingest fails?
    → GraphMerge sees StorageError from mirror upsert
  WHY StorageError?
    → INSERT raises 23505 on idx_*_embeddings_legacy_vector_id
  WHY unique violation?
    → two rows share (workspace_id, legacy_vector_id) with different FKs
  WHY different FKs for one lid?
    → resolve maps same logical name to distinct entity/rel UUIDs
       (alias display vs normalized, unordered index, dual create paths)
  WHY ON CONFLICT did not absorb?
    → conflict target is only (model_id, fk); legacy unique is a second index
```

## Normative absorb path

```ascii
  FOR EACH family (entity | relationship | report):

    1) UPDATE typed SET legacy_vector_id = COALESCE(legacy, $lid)
         WHERE (model_id, fk) matches AND legacy IS NULL
         -- stamp-once for existing PK rows

    2) INSERT new PK rows
         ON CONFLICT DO NOTHING   -- NO conflict_target
         -- absorbs PK duplicates AND legacy unique collisions

    3) Report absorbed_legacy_collisions (observability)
         NEVER return Err for idx_*_legacy_vector_id
```
