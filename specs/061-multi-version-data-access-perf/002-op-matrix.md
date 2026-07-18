# SPEC-061 — Op × proof matrix

Every catalog **OK** request-path op maps to a proof test. All tests run on **pg16 / pg17 / pg18** via `run_data_access_perf_matrix.sh`.

| Family | Op | Scale | SLO | EXPLAIN | Proof test | Matrixed |
|--------|-----|-------|-----|---------|------------|----------|
| KV | `upsert` | 1k | p95 &lt;100ms | Index | `e2e_spec060_ingest_stage_perf` + `e2e_spec061_kv_access_perf` | yes |
| KV | `get_by_ids` | 1k | p95 &lt;50ms | Index | `e2e_spec061_kv_access_perf` | yes |
| KV | `delete` | 1k | p95 &lt;100ms | Index | `e2e_spec061_kv_access_perf` | yes |
| KV | `keys_with_prefix` | 1k keys | p95 &lt;100ms | Index | `e2e_spec061_kv_access_perf` | yes |
| KV | `count` (stats) | — | &lt;20ms | — | `e2e_spec061_kv_access_perf` | yes |
| Vector | `query_filtered` | 2k / 50k | p95 &lt;100ms / &lt;500ms | HNSW | `e2e_spec054_*` | yes |
| Vector | `query` unfiltered | 10k | p95 &lt;100ms | HNSW | `e2e_spec061_vector_unfiltered_ann` | yes |
| Vector | `text_search_filtered` | ≥10k | p95 &lt;200ms | GIN | `e2e_spec060_fts_perf_explain` | yes |
| Vector | `upsert` / `upsert_report_created` | 1k | p95 &lt;500ms | — | `e2e_spec060_ingest_stage_perf` | yes |
| Vector | `delete` / compensate | K=1k | p95 &lt;500ms | — | `e2e_spec060_compensate_retract_perf` | yes |
| Vector | halfvec A/B | 2k | recall@20≥0.99; p95≤1.25× | — | `e2e_spec059_halfvec_perf_recall` | yes |
| Graph | `get_nodes_batch` | 100 | &lt;50ms | UNIQUE | `e2e_spec054_age_pgvector_perf` | yes |
| Graph | `get_incident_edges_batch` | ≥5k edges | p95 &lt;100ms | Bitmap/Index | `e2e_spec060_age_expand_perf` | yes |
| Graph | `node_counts_by_source_prefixes` | 20 prefixes | &lt;200ms | GIN | L1-a in `e2e_spec054_age_pgvector_perf` | yes |
| Graph | `node_degrees_batch` | 1k | p95 &lt;100ms | Index | `e2e_spec061_degrees_batch_perf` | yes |
| Graph | `upsert_nodes_batch` | 500 | &lt;500ms | — | Q3 / ingest stage | yes |
| Graph | `upsert_edges_batch` | 1k | &lt;500ms | — | `e2e_spec061_edge_upsert_perf` | yes |
| Graph | `delete_nodes_batch` | K=1k | &lt;500ms | — | compensate | yes |
| Query | Mix/Hybrid arms (Postgres) | seeded | documented p95 | — | `e2e_spec061_query_engine_postgres_arms` | yes |
| List | documents reconcile | — | &lt;500ms | — | `e2e_spec054_documents_list_perf` | yes |
| Stress | concurrent ANN/FTS/expand/Mix | default: 10k/64 (ANN), Mix 80; **prod**: 50k/1536, Mix 5k | pg16 N=8 ≤2×; pg17/18 N=16 ≤1.5× | — | `e2e_spec061_stress_concurrent_*` | yes |
| Stress | pool saturation | clients=16, pool=5, 2k rows | p95 &lt;2s (queue OK) | — | `e2e_spec061_stress_pool_saturation` | yes |

## FORBIDDEN (contracts only)

| API | Contract |
|-----|----------|
| `get_all_nodes` / `get_all_edges` | `contract_spec060_forbidden_request_path` |
| Unbounded `keys()` | same |
| Exact `COUNT(*)` on request path | prefer stats / `*_fast` |
