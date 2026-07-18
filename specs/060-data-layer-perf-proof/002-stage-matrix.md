# SPEC-060 — Stage → complexity → proof matrix

SSOT mapping request-path stages to [005-query-complexity-catalog](../054-fix-bugs-17/005-query-complexity-catalog.md).

| Stage | Ops | Complexity | Proof test |
|-------|-----|------------|------------|
| Ingest KV upsert | `upsert` UNNEST | O(K log N) | `e2e_spec060_ingest_stage_perf` |
| Ingest vector upsert | `upsert_report_created` | O(K log N)+HNSW | `e2e_spec060_ingest_stage_perf` |
| Ingest AGE node/edge | native ON CONFLICT | O(K log N) | Q3 (`e2e_spec054_age_pgvector_perf`) + `contract_spec060_native_writes` |
| Compensate / retract | delete by id/doc | O(K log N) | `e2e_spec060_compensate_retract_perf` |
| Query Naive ANN | `query_filtered` chunk | ~O(ef log N) | Q1-c / Q1-d |
| Query Local/Global | entity/rel ANN + expand | ANN + O(frontier) | `e2e_spec060_age_expand_perf` + `e2e_spec060_query_arm_wall_perf` |
| Query FTS arm | `text_search_filtered` | O(log N + cand) | `e2e_spec060_fts_perf_explain` |
| Mix/Hybrid arms | 3× above + semaphore | bounded concurrency | arm histograms + `e2e_spec059_arm_concurrency_load` |
| List/reconcile | `node_counts_by_source_prefixes` | O(D×probes log N) | L1-a |

## FORBIDDEN on request path

| API | Tag | Contract |
|-----|-----|----------|
| `get_all_nodes` / `get_all_edges` | FORBIDDEN | `contract_spec060_forbidden_request_path` |
| Unbounded `KVStorage::keys()` | ADMIN | same (allowlist admin/recovery modules) |
| Exact `COUNT(*)` on request path | ADMIN | prefer `*_fast` / stats |

## Legend

- **OK** — allowed on HTTP/query/ingest request path
- **ADMIN** — admin / clear / rebuild / offline only
- **FORBIDDEN** — must never run on request path
