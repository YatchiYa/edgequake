# SPEC-066 ceiling remasure — RUN_NOTES

**Date:** 2026-07-18  
**Host:** macOS, **128 GB** RAM  
**Profile:** pg18 (ephemeral via `scripts/eq_ephemeral_pg.sh`)  
**Shape:** Wave-2 only — `halfvec` + `EDGEQUAKE_HNSW_PARTIAL_BY_WORKSPACE=1` + column filter; global HNSW dropped in harness (battle shape)

## Host residency

| Knob | G1 | L2 500k | SEEK |
|------|----|---------|------|
| `shared_buffers` | 2–4 GB | **4 GB** | 4 GB |
| `shm_size` | 8g | **8g** | 8g |
| `maintenance_work_mem` | 2 GB | **8 GB** | 4 GB |

## Graph G1 (separate axis)

| Metric | Value |
|--------|-------|
| nodes | 100 000 |
| degrees sample | 1 000 ids × 20 samples |
| degrees p95 | **~11.8 ms** (SLO &lt;100 ms) |
| seed wall | ~7.7 s |
| Artifact | [`eq-ceiling-pg18-G1-0.jsonl`](eq-ceiling-pg18-G1-0.jsonl) |
| Result | **GREEN** — entities proven at G1 (not from vector N) |

## Wave-2 ANN ladder

### L2 = 500 000 rows @1536

| Metric | Value |
|--------|-------|
| seed | ~122 s |
| partial HNSW create | ~7.6 s |
| EXPLAIN | `uses_partial=true` (workspace partial Index Scan) |
| single p95 | **~2.3 ms** (Q1-d &lt;500 ms **pass**) |
| concurrent N=16 p95 | **~23 ms** (absolute Q1-d **pass**) |
| recall@20 vs exact (default ef) | **~0.31** (**FAIL** gate 0.99) |
| Artifact | [`eq-ceiling-pg18-L2-500000.jsonl`](eq-ceiling-pg18-L2-500000.jsonl) |
| Result | **LATENCY green / RECALL cliff** — do **not** promote L2 as full Q1-d+recall floor |

### SEEK = 250 000 rows @1536

| Metric | Value |
|--------|-------|
| seed | ~64 s |
| EXPLAIN | `uses_partial=false` — planner chose **Sort** (filter+exact), ~211 ms |
| recall@20 (ANN-relative + exact) | **1.00** (exact path) |
| single p95 | **~196 ms** (Q1-d pass) |
| concurrent N=16 p95 | **~1085 ms** (**FAIL** absolute Q1-d) |
| Artifact | [`eq-ceiling-pg18-SEEK-250000.jsonl`](eq-ceiling-pg18-SEEK-250000.jsonl) (JSONL later overwritten by SPEC-067 remasure) |
| Result | **Concurrent cliff** at 250k when planner prefers Sort; do not promote |

**SPEC-067 follow-on:** with session-local planner bias, SEEK remasure got `uses_partial=true`, concurrent ~21 ms (green), but **recall@20 ~0.56** — still not promoted. See [`specs/067-ops-real-floors/e2e/artifacts/RUN_NOTES.md`](../../../067-ops-real-floors/e2e/artifacts/RUN_NOTES.md).

### Remasure N=100 000 (methodology check)

| Metric | Value |
|--------|-------|
| EXPLAIN | `uses_partial=false` (Sort) on this host/stats draw |
| single p95 | ~76 ms |
| concurrent p95 | ~476 ms (absolute Q1-d **pass**; relative 1.5× warm miss — documented) |
| recall@20 | **1.00** |
| Artifact | [`eq-ceiling-pg18-N100000-100000.jsonl`](eq-ceiling-pg18-N100000-100000.jsonl) |
| Result | Confirms `highest_green_N=100000` under absolute concurrent gate |

### Ceiling fields (product gate = single Q1-d ∧ recall@20 ≥0.99 ∧ concurrent absolute)

| Field | Value | Notes |
|-------|-------|-------|
| `highest_green_N` | **100000** | SPEC-064 Wave-2 full gate; L2 latency-only does not promote |
| `first_fail_N` | **250000** | SEEK concurrent absolute Q1-d miss (earlier than L2 recall cliff) |
| Host class | 128 GB RAM / `shared_buffers=4GB` | See residency table |

## DiskANN

**Out of scope** — failures are recall (L2 HNSW) / concurrent (SEEK Sort), not a hang/FORBIDDEN single-query latency cliff. DiskANN study not opened.

## Commands

```bash
EQ_CEILING_STEP=G1 make ceiling-proof
EQ_CEILING_STEP=L2 make ceiling-proof
EQ_CEILING_STEP=SEEK EDGEQUAKE_CEILING_ROWS=250000 make ceiling-proof
make product-limits-check
```
