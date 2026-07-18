# SPEC-059 — Test matrix

| Case | Test |
|------|------|
| Second upsert_report_created empty | `merger::tests::spec059_upsert_report_created_second_call_empty` |
| Concurrent xmax insert detection | `spec059_upsert_report_created_postgres` |
| Cancel facade unindexes | `e2e_spec059_cancel_retract` |
| Orphan retract keeps neighbors | `e2e_spec059_orphan_retract` |
| Concurrent source_ids union | `spec059_concurrent_source_ids_race_postgres` |
| HNSW ef_construction=64 new index | `e2e_spec059_hnsw_indexdef_ef64` |
| halfvec p95 ≤1.25× full, recall@20≥0.99 | `e2e_spec059_halfvec_perf_recall` |
| Mix arm concurrency bound | `e2e_spec059_arm_concurrency_load` |
| Filtered ANN p95 @50k (regression) | `e2e_spec054_mix_scale_perf` |

## Commands

```bash
cd edgequake
export DATABASE_URL="$(cat /tmp/edgequake-db-url)"
# optional CI: export EDGEQUAKE_REQUIRE_POSTGRES_TESTS=1

cargo test -p edgequake-pipeline --lib spec059_upsert_report_created
cargo test -p edgequake-storage --features postgres --test spec059_upsert_report_created_postgres
cargo test -p edgequake-storage --features postgres --test spec059_concurrent_source_ids_race_postgres
cargo test -p edgequake-api --test e2e_spec059_cancel_retract
cargo test -p edgequake-api --test e2e_spec059_orphan_retract
cargo test -p edgequake-storage --features postgres --test e2e_spec059_hnsw_indexdef_ef64
cargo test -p edgequake-storage --features postgres --test e2e_spec059_halfvec_perf_recall -- --nocapture
cargo test -p edgequake-query --test e2e_spec059_arm_concurrency_load
cargo test -p edgequake-storage --features postgres --test e2e_spec054_mix_scale_perf -- --nocapture
```

## SLOs

| Gate | Budget |
|------|--------|
| Filtered ANN p95 @ ≥50k | &lt; 500ms |
| halfvec p95 vs full | ≤ 1.25× |
| halfvec recall@20 vs full | ≥ 0.99 |
| Mix arm peak in-flight | ≤ `EDGEQUAKE_QUERY_ARM_CONCURRENCY` (default 4) |
