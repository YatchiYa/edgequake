# SPEC-064 battle notes — pg18 release @100k/1536

**Command:** `make ann-scale-battle`  
**Artifacts:** `eq-battle-pg18.jsonl`, `WAVE0_EXPLAIN.md`, `eq-battle-pg18-cargo.log`  
**Date:** 2026-07-18

## Arm results (single-client p95)

| Arm | Storage | Index | single p95 | recall@20 | Q1-d |
|-----|---------|-------|------------|-----------|------|
| `full_default` | vector | global (planner → btree+exact) | ~56ms warm / ~1483ms cold | 1.0 (ref) | warm pass / cold miss |
| `halfvec_default` | halfvec | global | ~2376ms (coldish) | **0.99** | miss |
| `halfvec_partial_ws` | halfvec | partial WS + column filter | **~65ms** | **0.99** | **pass** |
| `guc_grid` best | halfvec table | same | **~61ms** (`ef=40,max=20k,mem=1`) | — | pass |

## Stress (N=16)

Relative 1.5× vs a very fast single is harsh; absolute concurrent p95 for partial ≈ **408ms** (under Q1-d 500ms, over 1.5× of 65ms). Document as: concurrent OK under Q1-d absolute; tighten pool/host before claiming 1.5× relative.

## Wave 3 GUC knee

Grid `ef∈{40,80,120}` × `max_scan_tuples∈{5k,20k,50k}` × `scan_mem_multiplier∈{1,2}` on winning table:

- Best: **ef=40, max_scan_tuples=20000, scan_mem_multiplier=1** (~61ms)
- Worst in grid: ef=120 / max=50k / mem=2 (~95ms)
- **Do not** raise defaults blindly — code defaults already near-optimal for this plan shape

## Promote decisions

| Change | Decision |
|--------|----------|
| Prod default `EDGEQUAKE_VECTOR_STORAGE=halfvec` | **No silent flip** — recall OK, but global halfvec arm not faster alone |
| `EDGEQUAKE_HNSW_PARTIAL_BY_WORKSPACE=1` | **Opt-in for hot workspaces** — L1 Q1-d green with halfvec + partial |
| Search GUC defaults | **Keep** (`ef≈4×k`, `max_scan_tuples=20000`); optional ops tip `EF_SEARCH=40` |
| SPEC-063 L1 envelope | **Update** — Q1-d achievable on pg18 with Wave2 shape; cold exact scan still a cliff |

## Non-regression

`EQ_PERF_PROFILES=pg18 make data-access-perf-matrix-prod` — **green** 2026-07-18 (post-battle).
