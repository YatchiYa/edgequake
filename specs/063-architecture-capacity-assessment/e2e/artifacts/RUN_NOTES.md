# SPEC-063 capacity ladder — RUN NOTES

**Date:** 2026-07-18  
**Command:** `EDGEQUAKE_CAPACITY_LADDER=L1 make data-access-perf-capacity-ladder`  
**Profile:** pg18 (`edgequake-postgres:local`), AGE 1.7 / pgvector 0.8.5  
**Build:** `cargo test --release`  
**Wall:** ~3.4 min (seed ~30s + HNSW create ~79s + query)

## L1 result (100k vectors @1536)

| Op | p95_ms | pass | Notes |
|----|--------|------|-------|
| `capacity_ladder_ensure_ann_index` | ~78 780 | yes | deferred heap load then CREATE INDEX |
| `capacity_ladder_filtered_ann_single` | **~1470** | **no** | Q1-d SLO is 500ms — **measured cliff** |
| `capacity_ladder_filtered_ann_stress` | ~355 | yes | N=16 ≤1.5× single (after warm); not a Q1-d promotion |

**Claim promotion:** do **not** mark “100k @1536 supported at Q1-d”. L1 is a completed measurement showing filtered ANN single-client ~1.5s on this host class. Proven envelope remains **50k @1536** (SPEC-061/062 prod matrix).

Artifact: [`eq-capacity-pg18-L1.jsonl`](eq-capacity-pg18-L1.jsonl)

## Re-run

```bash
EDGEQUAKE_CAPACITY_LADDER=L1 make data-access-perf-capacity-ladder
# L2/L3 (long): EDGEQUAKE_CAPACITY_LADDER=L2 make data-access-perf-capacity-ladder
```
