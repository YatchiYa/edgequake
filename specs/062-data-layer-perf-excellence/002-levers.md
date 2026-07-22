# SPEC-062 — Levers (ranked)

| Rank | Lever | Expected impact | Risk |
|------|-------|-----------------|------|
| 1 | Denormalized `eq_source_id` / `eq_target_id` / `eq_node_id` + btree; stop per-row `agtype_to_json` on write/degrees | High on pg16 writes | Med (migration + dual-read) |
| 2 | Complete `PERF_REPORT` coverage + release matrix + 2× cross-major gate | Honesty / CI trend | Low |
| 3 | HNSW `ef_construction` / batch tuning; halfvec greenfield | Cut ingest ~390ms wall | Med (recall / REINDEX) |
| 4 | Stress N=16; stress ≤1.5× on pg17/18 | Concurrency realism | Low–med |
| 5 | Skip `eq_merge_graph_properties` when full replace is safe | Med on conflict updates | Low |

Deferred: Mix RRF, DiskANN, 1M PR soak, Criterion as release gate.
