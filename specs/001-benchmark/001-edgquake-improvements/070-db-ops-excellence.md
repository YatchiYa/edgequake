# 070 — Database Operations Excellence (Fact Gates)

**Status:** Implemented  
**Date:** 2026-07-22  
**Law:** Schema ≠ request path; measure or don’t claim; O(1)/O(log N)/O(probes) on hot path

## Problem

Query/ANN and native upsert already have SPEC-054 budgets, but coverage was uneven: some migrations were sqlx-once only, vector index builds lacked DDL session GUCs, and delete-under-load lacked a fact gate proving zero mid-task DDL + GIN plans. Judgments risked becoming vibe-driven.

## Laws

| Law | Meaning |
|-----|---------|
| Schema ≠ request path | DDL only at boot/reconcile/operator REINDEX |
| Hot-path complexity | O(1) catalog, O(log N) btree/UNIQUE, O(k log N) ANN, O(probes) GIN JOIN |
| Fail closed | Missing schema/index → clear error; no mid-request heal |
| Measure or don’t claim | Every budget has test ID + EXPLAIN/wall |
| One SSOT | `support/NNN/` boot reconcile; `search_tuning` query GUCs; DDL sessions separate |

Platform pins: PG16/17/18, pgvector ≥0.8.2 (prefer 0.8.5), AGE 1.7+ on modern tags. Cross-ref: [SPEC-054 July 2026](../../../054-fix-bugs-17/006-july-2026-alignment.md), [SPEC-069](./069-reliable-delete-ddl-off-hotpath.md).

## Op-family scorecard

| Family | Primary code | Complexity | Budget | Gate test | Status |
|--------|--------------|------------|--------|-----------|--------|
| Boot M083/M092 | `migration_bootstrap`, `support/083\|092` | O(graphs) catalog | B1-a &lt;2s when ready | `e2e_spec054_*` / contract 069 | keep |
| Boot M086 BFS edges | `support/086/apply.sql` | O(graphs) catalog | every boot; IF NOT EXISTS | `contract_spec070_*` / m086 unit | **closed** |
| Graph ensure_indexes | `graph_lifecycle.rs` | O(1) when verified | zero DDL during delete/ingest | `contract_spec069_*` | keep |
| Native node/edge upsert | `nodes_ops/mutate.rs`, `edges_ops.rs` | O(batch log N) | Q3-b &lt;500ms @500 | `e2e_spec054_age_pgvector_perf` | keep |
| Source-prefix discovery | `scan_ops.rs` | O(probes) | no SeqScan; &lt;2s warm | `contract_source_prefix_discovery_gin` + `e2e_spec070_delete_no_ddl` (Node) + `e2e_spec071_edge_source_prefix_gin` (EDGE) | **closed by 071** |
| Cascade delete | `document_graph_cascade.rs` | O(affected) | zero mid-task DDL; terminal | `e2e_spec070_delete_no_ddl` | **closed** |
| Filtered HNSW | `vector/search_tuning.rs` | O(k log N) | Q1-c | `e2e_spec054_*` / 075 | keep |
| Expand / degrees | `query_ops/expand.rs` | O(degree) | Q2 | `e2e_spec060/061` | keep |
| FTS | `vector/fts.rs` | O(log N) GIN | no SeqScan | `e2e_spec060_fts_perf_explain` | keep |
| Vector index build | `vector/ddl.rs` | O(N log N) build | DDL session GUCs | `contract_spec070_vector_ddl_session` | **closed** |
| KV prefix/suffix | `kv.rs` | O(log N) + reverse-suffix | LIMIT safety cap on legacy APIs | `contract_spec070_ops_audit` (+ e2e_spec061) | **closed** |
| Task claim SKIP LOCKED | M088 + worker | O(1) claim | no lock pile-up | `contract_spec070_ops_audit` | audited |
| PDF / originals / mm | `pdf_*`, `original_*`, `mm_asset_*` | O(1) by id / FK cascade | no full-table scan on request | `contract_spec070_ops_audit` | audited |
| Conversations | `conversation.rs` | O(log N) keyed | scoped list limits | `contract_spec070_ops_audit` | audited |

### Forbidden on request path

- Unbounded `get_all_nodes` / `get_all_edges` (SPEC-006)
- Mid-task `ALTER TABLE` / `DROP TRIGGER` / `CREATE TRIGGER` (SPEC-069/070)
- Unconditional O(N) UNIQUE dedup when arbiter index valid (SPEC-054)
- Leading-`%` LIKE without reverse-suffix index (SPEC-011)
- Criterion in-memory benches as release gates

### Session GUCs

| Path | GUCs |
|------|------|
| Graph query | `statement_timeout = EDGEQUAKE_GRAPH_QUERY_TIMEOUT_SECS` (default 15s) |
| Graph / AGE DDL | `statement_timeout=0`, `lock_timeout=5s` (`setup_age_ddl_session`) |
| Vector index DDL | same + `maintenance_work_mem=EDGEQUAKE_INDEX_MAINTENANCE_WORK_MEM` (default `256MB`) |
| Filtered ANN | `SET LOCAL` ef_search / iterative_scan (SPEC-054/080) |

### AGE 1.7 note

AGE 1.7+ may auto-create id/start_id/end_id indexes. EdgeQuake still requires property expression indexes, `source_ids` GIN, eq_* arbiters, and BFS `idx_edge_source_id` / `idx_edge_target_id` (M086 every boot).

## Changes (this pack)

1. Every-boot `reconcile_migration_086` (`migrations/support/086/apply.sql`).
2. `setup_vector_ddl_session` before HNSW/IVFFlat/FTS index CREATE in `vector/ddl.rs`.
3. Live `e2e_spec070_delete_no_ddl` — concurrent cascade, zero DDL statements, GIN EXPLAIN, wall budget.
4. Source contracts for M086 wiring, vector DDL GUCs, ops-family anti-patterns.
5. Doc refresh: this scorecard, SPEC-054 index checklist (eq_* primary), `migrations/NOTES.md`.

## Verify

```bash
# From edgequake/
cargo test -p edgequake-storage --test contract_spec070_vector_ddl_session
cargo test -p edgequake-storage --test contract_spec070_ops_audit
cargo test -p edgequake-api --test contract_spec070_m086_reconcile
cargo test -p edgequake-api --features postgres --lib m086_apply_sql
cargo test -p edgequake-storage --test contract_spec069_ddl_off_hotpath
cargo test -p edgequake-storage --test contract_source_prefix_discovery_gin

# Live (DATABASE_URL required):
cargo test -p edgequake-storage --test e2e_spec070_delete_no_ddl -- --ignored
# or without --ignored if the suite auto-skips when DATABASE_URL unset

cargo fmt -p edgequake-storage -p edgequake-api
```

## Success criteria

1. Scorecard lists every op family with complexity + budget + test ID.
2. New AGE graphs get M086 BFS indexes at boot without waiting for ensure_indexes race.
3. Vector index builds apply DDL-safe session GUCs (contract-locked).
4. Concurrent delete e2e proves no mid-task DDL + GIN discovery plan.
5. SPEC-054 checklist prefers eq_* arbiters; NOTES.md max version current.
