# 08 — Test protocol

Fail-closed: `EDGEQUAKE_REQUIRE_POSTGRES_TESTS=1` panics on skip (same gate as SPEC-129).

Assert `pg_get_indexdef` still unique-indexes `(workspace_id, legacy_vector_id)` so a dropped index cannot fake green.

| ID | Artifact | Proves |
|----|----------|--------|
| T-377-0 | `contract_spec120_legacy_vector_id_race` | Raw `UPDATE ... SET legacy_vector_id` on loser PK → SQLSTATE **23505** |
| T-377-1 | same | Loser NULL-lid PK `upsert_batch` → `Ok`, `absorbed_legacy_collisions >= 1`, one owner; **retry** `Ok` |
| T-377-2 | same | Relationship twin |
| T-377-3 | `e2e_spec136_durable_lid_retry` | Sequential merger on durable fixture; `errors == 0` twice |
| T-377-4 | same e2e | Winner keeps lid; loser PK does not steal it |

```bash
export DATABASE_URL=postgresql://edgequake:edgequake@localhost:5432/edgequake
export EDGEQUAKE_REQUIRE_POSTGRES_TESTS=1
cargo test -p edgequake-storage --features postgres \
  --test contract_spec120_legacy_vector_id_race -- --test-threads=1
cargo test -p edgequake-api --features postgres \
  --test e2e_spec136_durable_lid_retry -- --test-threads=1
```

Existing SPEC-120 concurrent tests stay: they prove INSERT absorb. These prove the UPDATE hole and retry.
