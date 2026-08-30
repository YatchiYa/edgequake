# 02 — Cross-ref matrix

| This pack | Other SSOT | Contract |
|-----------|------------|----------|
| LAW-139-1 | [`fleet_embedding_backfill.rs`](../../edgequake/crates/edgequake-storage/src/migration_engine/fleet_embedding_backfill.rs) · [`conflict_dedupe.rs`](../../edgequake/crates/edgequake-storage/src/migration_engine/conflict_dedupe.rs) | E2E-139-01/02, U-139-DEDUP |
| LAW-139-1 sibling | [`storage_impl.rs`](../../edgequake/crates/edgequake-storage/src/adapters/postgres/vector/storage_impl.rs) QW2 | existing vector upsert tests |
| LAW-139-2 | [`fleet_provenance_stamp.rs`](../../edgequake/crates/edgequake-storage/src/migration_engine/fleet_provenance_stamp.rs) | SPEC-111 stamp e2e |
| LAW-139-3 | [`verify.rs`](../../edgequake/crates/edgequake-storage/src/migration_engine/verify.rs) · [`coverage.rs`](../../edgequake/crates/edgequake-storage/src/migration_engine/coverage.rs) · [126](../../edgequake/migrations/126_spec091_vector_drop.sql) | E2E-139-03 |
| LAW-139-4 | [`lease.rs`](../../edgequake/crates/edgequake-storage/src/migration_engine/lease.rs) `reclaim_verify_failed_jobs` | E2E-139-04 |
| LAW-139-5 | [`runner.rs`](../../edgequake/crates/edgequake-storage/src/migration_engine/runner.rs) `run_engine` | E2E-139-06 |
| LAW-139-6 | remainder jobs · [119](../../edgequake/migrations/119_spec091_artifact_backfill.sql) · [117](../../edgequake/migrations/117_spec091_dedup_backfill.sql) · [122](../../edgequake/migrations/122_spec091_shell_backfill.sql) | E2E-139-05 |
| LAW-139-7 | 125/126/131 unchanged | existing advisor≡SQL contracts |
| Normalize | [`EntityNameIndex`](../../edgequake/crates/edgequake-storage/src/migration_engine/coverage.rs) · LAW-111-6 | E2E-111 normalize + 139-01 |
| Consent CLI | SPEC-137 | not re-tested here |
| Equality env | `EDGEQUAKE_MIGRATION_VERIFY_EQUALITY` | unit `passes()` + 111 tests |
| Pack template | [SPEC-110](../110-migration-issue/) · [SPEC-137](../137-issue-migration-25-to-26/) | structure |

## Divergence rule

If this matrix and a code comment disagree, **code + contract test** win. Update
this file in the same PR as the code change.
