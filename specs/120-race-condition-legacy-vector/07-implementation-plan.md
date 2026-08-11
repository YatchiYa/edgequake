# 07 — Implementation Plan

## Principles

- **DRY:** one absorb helper for entity / relationship / report
- **SOLID:** conflict policy in storage adapter; merge call site unchanged
- **First principles:** bookkeeping must not fail content (LAW-120-1)
- **Test first:** force dual-FK same lid before claiming fixed

## Phase A — Absorb upsert (P0)

1. Extend `MirrorLegacyReport` / `UpsertReport` with `absorbed_legacy_collisions`.
2. Extract DRY module `fleet_legacy_absorb.rs`:
   - UPDATE stamp-once by PK
   - INSERT `ON CONFLICT DO NOTHING` (no target)
   - Count absorbed lid-bearing FK misses
   - Table/FK from `EmbeddingFamily::{typed_table,typed_fk_column,typed_fk_is_uuid}`
3. `PgFleetEmbeddingIndex::upsert_batch` delegates to the module (SRP).
4. Log warn when absorbed > 0.

## Phase B — Resolve hygiene (P1)

1. `load_entity_name_index` + `load_entity_name_index_pool`: add  
   `ORDER BY created_at ASC, id ASC`.
2. Document alias merge as non-goal (SPEC-083).

## Phase C — Tests

1. `contract_spec120_legacy_vector_id_race.rs` — dual FK concurrent upsert.
2. Stamp-once + multi-WS + rel/report families.
3. `e2e_spec120_concurrent_mirror_same_entity.rs` — mirror path.

## Phase D — Docs / GitHub

1. Comment on #374 with findings + SPEC-120 link + nuance.
2. Update README status + acceptance.
3. Close #374 only when acceptance green.

## Edge-case matrix

| ID | Case | Mitigation | Test |
|----|------|------------|------|
| EC-01 | Dual FK same lid same WS | Targetless DO NOTHING | T1 |
| EC-02 | Concurrent upserts | Same absorb | T1 tokio join |
| EC-03 | Multi-WS same lid | Unique includes workspace_id | T3 |
| EC-04 | Stamp-once non-null lid | UPDATE WHERE NULL | T4 |
| EC-05 | Null / empty lid | NULLIF; unique partial skips NULL | unit/contract |
| EC-06 | Empty batch | Early Ok | existing |
| EC-07 | Mixed collide + fresh rows | Unnest batch; DO NOTHING per row | T1 mixed |
| EC-08 | Relationship family | Same helper | T5 |
| EC-09 | Report family | Same helper | T5 |
| EC-10 | Mirror resolve miss | Existing miss report (not absorb) | SPEC-098 |
| EC-11 | Absorb must not GraphMerge | Upsert returns Ok | T2 |
| EC-12 | Winner compensation | No Err → no compensate | T2 |
| EC-13 | Exact-name sink race | Already ON CONFLICT | out of scope assert |
| EC-14 | Alias display vs normalized | P1 order + SPEC-083 follow-up | note |

## Rollout

1. Land docs + code + tests (no migration).
2. Deploy; absorbed collisions appear only in logs.
3. Close #374 after acceptance.
