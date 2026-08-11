# 08 — Test Protocol

## Named tests

| ID | Artifact | Assert |
|----|----------|--------|
| T1 | `edgequake-storage` `contract_spec120_legacy_vector_id_race.rs` | Dual-FK concurrent upsert absorb; stamp-once; multi-WS; rel+report; mixed batch |
| T2a | `edgequake-storage` `e2e_spec120_concurrent_mirror_same_entity.rs` | Concurrent / losing-FK `mirror_legacy_batch` → Ok; one lid owner |
| T2b | `edgequake-api` `e2e_spec120_concurrent_merger_mirror.rs` | Concurrent merger **entity** + **relationship** → `errors==0`, one spine, one lid each |
| T3 | Multi-WS in T1 + 091 | Same lid in two workspaces both succeed |
| T4 | Stamp-once in T1 | Non-null lid not overwritten |
| T5 | Rel + report in T1 | Same absorb policy |

**Not in scope for this mission:** full HTTP upload + background worker dual-doc race (heavier; merger path is the intentional bound).

## How to run

```bash
export DATABASE_URL=postgresql://edgequake:edgequake_secret@localhost:5432/edgequake
export EDGEQUAKE_REQUIRE_POSTGRES_TESTS=1
cargo test -p edgequake-storage --features postgres \
  --test contract_spec120_legacy_vector_id_race \
  --test e2e_spec120_concurrent_mirror_same_entity
cargo test -p edgequake-api --features postgres \
  --test e2e_spec120_concurrent_merger_mirror
```

## Pass criteria

1. No `idx_*_legacy_vector_id` unique_violation propagated to callers.
2. Exactly one typed owner per test lid in the workspace.
3. Multi-WS duplicate lids allowed.
4. Merger concurrent path: both `stats.errors == 0`.
5. Existing `contract_spec091_fleet_mirror_fk` remains green.
