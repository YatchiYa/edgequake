# SPEC-059 — Data-layer integrity completion + verified perf

Closes SPEC-058 residual holes: compensate TOCTOU, retract-on-all-cancel-paths, orphan/stuck unindex, concurrent `source_ids` race proof, verified HNSW/halfvec/Mix gates, Prometheus/queue-metrics.

| Wave | Deliverable |
|------|-------------|
| 1 | `upsert_report_created` (`xmax=0`) + merger artifacts |
| 2 | Cancel facade / pipeline / PDF / stuck / reprocess retract |
| 3 | Orphan incomplete-doc janitor retract (`EDGEQUAKE_ORPHAN_RETRACT_ON_RECOVER`) |
| 4 | Concurrent `source_ids` race e2e (M090) |
| 5 | halfvec A/B, HNSW ef64 indexdef, Mix arm load |
| 6 | Metrics + this pack + data-layer.md |

See [001-first-principles.md](001-first-principles.md), [002-test-matrix.md](002-test-matrix.md).
