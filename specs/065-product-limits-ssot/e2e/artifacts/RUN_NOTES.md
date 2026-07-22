# SPEC-065 remasure notes — Wave-2 productization + limits SSOT

**Date:** 2026-07-18  
**Depends on:** SPEC-063 envelope, SPEC-064 battle evidence  
**SSOT:** [`docs/product-limits.md`](../../../../docs/product-limits.md)

## Gates

| Gate | Command | Result |
|------|---------|--------|
| Policy unit | `cargo test -p edgequake-storage --lib --features postgres from_env_clamps` | green |
| Lifecycle e2e | `e2e_spec065_partial_hnsw_lifecycle` on ephemeral pg18 | green (env mutex; parallel-safe) |
| Battle | `EQ_PERF_PROFILES=pg18 make ann-scale-battle` | **green** |
| Prod matrix | `EQ_PERF_PROFILES=pg18 make data-access-perf-matrix-prod` | **green** |
| Honesty | `make product-limits-check` | green |

## Battle remasure (post-SPEC-065 wiring)

| Arm | single p95 | recall@20 | Q1-d |
|-----|------------|-----------|------|
| `full_default` | ~1577ms | 1.0 | miss (cold cliff) |
| `halfvec_default` | ~1426ms | **0.99** | miss |
| `halfvec_partial_ws` | **~63ms** | **0.99** | **pass** |
| `guc_grid` best | **~60ms** (`ef=120,max=20k,mem=1`) | — | pass |

Artifacts: `specs/064-filtered-ann-scale-battle/e2e/artifacts/eq-battle-pg18.jsonl` (copy under this dir).

## Prod matrix remasure (pg18)

- Command: `EQ_PERF_PROFILES=pg18 make data-access-perf-matrix-prod`
- Harness now uses `scripts/eq_ephemeral_pg.sh` with **2GB `shared_buffers` / 4g shm** for prod/large (SPEC-065 DRY + residency).
- Concurrent filtered ANN stress: **p95 ~167ms** (pass; wall ~5.6s) after residency fix.
- Prior flaky fails (~3s p95) were cold-cliff thrash under default Docker `shared_buffers` (~128MB) + host contention — not a Wave-2 logic regression.
- Stress harness also warms concurrent clients before the timed window.

Artifact: `eq-perf-pg18.jsonl` (this directory + SPEC-061 artifacts).

## Product decisions (unchanged)

- No silent `EDGEQUAKE_VECTOR_STORAGE=halfvec` default flip
- Partial HNSW remains opt-in: `EDGEQUAKE_HNSW_PARTIAL_BY_WORKSPACE=1`
- GUC defaults kept; `ef=40` ops tip only
- pg16/17: same physics; remasure on demand

## Lifecycle productization

- `query_filtered` → `ensure_hot_workspace_ann` when workspace present + flag on
- Shared tables only; dedicated `_ws_` tables skip
- Fail-closed if DDL claims success but catalog missing partial
- Row threshold: `EDGEQUAKE_HNSW_PARTIAL_MIN_ROWS` (default 1000)
