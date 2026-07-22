# SPEC-060 — SLOs and commands

## SLOs

| ID | Metric | Target |
|----|--------|--------|
| Q1-c | Filtered HNSW @2k | p95 &lt; 100ms |
| Q1-d | Filtered HNSW @50k | p95 &lt; 500ms |
| Q2-expand | Scoped incident edges @≥5k edges | p95 &lt; 100ms |
| Q-FTS | FTS @≥10k content_ref chunks | p95 &lt; 200ms |
| I-KV | KV upsert 1k rows | &lt; 100ms |
| I-VEC | Vector upsert 1k rows | &lt; 500ms |
| I-AGE | Native AGE upsert 500 nodes | &lt; 500ms |
| C-RET | Compensate/retract K=1k | &lt; 500ms |
| HV | halfvec recall@20 vs full | ≥ 0.99; p95 ≤ 1.25× |

## Commands

```bash
cd edgequake
export DATABASE_URL="$(cat /tmp/edgequake-db-url)"
export EDGEQUAKE_REQUIRE_POSTGRES_TESTS=1
export EDGEQUAKE_NATIVE_GRAPH_WRITES=1

# Contracts (no DB)
cargo test -p edgequake-storage --test contract_spec060_forbidden_request_path
cargo test -p edgequake-storage --test contract_spec060_native_writes

# Postgres gates
cargo test -p edgequake-storage --features postgres --test e2e_spec054_age_pgvector_perf -- --nocapture
cargo test -p edgequake-storage --features postgres --test e2e_spec054_mix_scale_perf -- --nocapture
cargo test -p edgequake-storage --features postgres --test e2e_spec059_halfvec_perf_recall -- --nocapture
cargo test -p edgequake-storage --features postgres --test e2e_spec059_hnsw_indexdef_ef64 -- --nocapture
cargo test -p edgequake-storage --features postgres --test e2e_spec060_fts_perf_explain -- --nocapture
cargo test -p edgequake-storage --features postgres --test e2e_spec060_age_expand_perf -- --nocapture
cargo test -p edgequake-storage --features postgres --test e2e_spec060_ingest_stage_perf -- --nocapture
cargo test -p edgequake-storage --features postgres --test e2e_spec060_compensate_retract_perf -- --nocapture
cargo test -p edgequake-query --test e2e_spec060_query_arm_wall_perf -- --nocapture
cargo test -p edgequake-query --test e2e_spec059_arm_concurrency_load -- --nocapture
```

Nightly CI: `.github/workflows/postgres-matrix-nightly.yml`

- `spec060-contracts` — FORBIDDEN grep + native-writes + arm-wall (no DB)
- `spec060-postgres-perf` — `EDGEQUAKE_REQUIRE_POSTGRES_TESTS=1` + 054/059/060 Postgres gates (soft-skip on missing DB fails the job)
- PR `postgres-integration` stays functional-only (latency suite stays nightly / `battle=true`)
