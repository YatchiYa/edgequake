# SPEC-069 dedicated mid-scale — RUN_NOTES

**Date:** 2026-07-18  
**Host:** macOS, **128 GB** RAM  
**Profile:** pg18 + AGE 1.8 / pgvector 0.8.5  
**Shape:** **Dedicated** `*_ws_*` table (= hot-set), halfvec, global HNSW (no partial)  
**Command:** `make dedicated-midscale`  
**Artifact:** [`eq-dedicated-midscale-pg18.jsonl`](eq-dedicated-midscale-pg18.jsonl)

## Ladder (clients=16, absolute Q1-d)

| rows | ef | single p95 | stress p95 | recall@20 | full_green |
|------|-----|------------|------------|-----------|------------|
| 100k | 80 | ~166 ms | **~3109 ms** | OK | **FAIL** concurrent |
| 100k | 240 | ~175 ms | **~2813 ms** | OK | **FAIL** concurrent |
| 125k | 80/240 | ~208–242 ms | **~3.7 s** | OK | FAIL |
| 150k | 80/240 | ~281–301 ms | **~4.6–4.7 s** | OK | FAIL |
| 200k | 80/240 | ~264–275 ms | **~6.1–6.2 s** | OK | FAIL |

EXPLAIN: `uses_hnsw=true`, `uses_sort=false` on all rungs (Buffers shared hit — not cold thrash).

## Contention matrix @100k ef=80 (diagnostic)

| clients | scan_mem | stress p95 | abs Q1-d |
|---------|----------|------------|----------|
| **4** | off / ×2 | ~279 / ~404 ms | **pass** |
| **8** | off / ×2 | ~1.4 s | fail |
| **16** | off / ×2 | ~3.1 s | fail |

`scan_mem_multiplier=2` did **not** restore clients=16.

## Promotion decision

| Field | Value |
|-------|-------|
| Promote 150k? | **No** |
| `highest_green_N` | **100 000** (unchanged — still from **shared+partial** Wave-2, SPEC-064/068) |
| `first_fail_N` | **250 000** (unchanged) |
| Dedicated concurrent unlock? | **No** — dedicated is **worse** under clients=16 than shared+partial @100k |

### Physics read

Dedicated tables correctly use HNSW and keep single-query Q1-d, but concurrent absolute collapses earlier than the shared+partial Wave-2 proof shape. Do **not** market dedicated tables as a mid-scale concurrent floor. Dedicated remains the right product path for **per-workspace dimension isolation**, not for raising ANN N.

## SPEC-070 DiskANN — **OPEN**

Exit criteria met:

1. Dedicated ladder archived through 150k+ with Wave-2 halfvec + residency class  
2. Full gate fails at **clients=16** after ef tip (240) + scan_mem tip  
3. **HNSW concurrent wall with recall green @150k on dedicated path** (and already at 100k dedicated)

SPEC-070 may revise the old hang-only DiskANN trigger to: topology study allowed when HNSW cannot clear concurrent absolute at N≥150k with recall green on dedicated tables (opt-in; no silent default; promote only from green).

See stub: [`specs/070-diskann-study/000-index.md`](../../../070-diskann-study/000-index.md).
