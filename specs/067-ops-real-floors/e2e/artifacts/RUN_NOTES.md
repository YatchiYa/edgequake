# SPEC-067 SEEK remasure — RUN_NOTES

**Date:** 2026-07-18  
**Host:** macOS, **128 GB** RAM  
**Profile:** pg18 (ephemeral)  
**Shape:** Wave-2 + **SPEC-067 session-local planner bias** (`enable_seqscan=off`, `random_page_cost=1.1` on columns-only filtered path)  
**Command:** `EQ_CEILING_STEP=SEEK EDGEQUAKE_CEILING_ROWS=250000 make ceiling-proof`

## SEEK = 250 000 rows @1536 (post planner bias)

| Metric | SPEC-066 (pre-bias) | SPEC-067 (this remasure) |
|--------|---------------------|---------------------------|
| EXPLAIN | `uses_partial=false` (Sort) | **`uses_partial=true`** (partial HNSW Index Scan) |
| single p95 | ~196 ms | **~3.0 ms** (Q1-d latency pass) |
| concurrent N=16 p95 | ~1085 ms (**FAIL** abs Q1-d) | **~20.8 ms** (**pass** abs + rel) |
| recall@20 ANN-relative | 1.00 (exact Sort path) | **~0.56** (**FAIL** gate 0.99) |
| Artifact | (superseded for SEEK) | [`eq-ceiling-pg18-SEEK-250000.jsonl`](eq-ceiling-pg18-SEEK-250000.jsonl) |

### Product gate

Gate = single Q1-d ∧ recall@20 ≥0.99 ∧ concurrent absolute.

| Field | Value | Notes |
|-------|-------|-------|
| `highest_green_N` | **100 000** (unchanged) | No promotion — 250k fails recall |
| `first_fail_N` | **250 000** (unchanged) | Failure mode shifted: concurrent Sort → **recall cliff on HNSW** |
| Promote 250k? | **No** | Latency green ≠ full Q1-d+recall floor |

## Interpretation

Planner bias achieved the perf goal (partial HNSW preferred; concurrent absolute green). Capacity claims stay honest: **do not** raise supported N beyond 100k.

## DiskANN

Still **out of scope** — not a hang/FORBIDDEN cliff; recall quality cliff on HNSW.
