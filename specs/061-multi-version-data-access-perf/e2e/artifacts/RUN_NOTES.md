# SPEC-061/062 multi-major matrix — RUN NOTES (prod stress remasure)

**Date:** 2026-07-18  
**Command:** `make data-access-perf-matrix-prod` (`EDGEQUAKE_PERF_RELEASE=1 EDGEQUAKE_PERF_SCALE=prod`)  
**Wall time:** ~28.6 min (all majors sequential)  
**Host:** local Docker (OrbStack), ephemeral `edgequake-perf-{profile}-$$`  
**Result:** **PASS** — `OK pg16`, `OK pg17`, `OK pg18`; cross-major compare exit 0

Also validated earlier: `EQ_PERF_PROFILES=pg18 EDGEQUAKE_PERF_SCALE=default make data-access-perf-matrix-release`.

## Extension pins (asserted)

| Profile | Image tag | AGE | pgvector |
|---------|-----------|-----|----------|
| pg16 | `edgequake-postgres:pg16` | 1.6.0 (≥1.6.0) | 0.8.5 |
| pg17 | `edgequake-postgres:pg17` | 1.7.0 (≥1.7.0) | 0.8.5 |
| pg18 | `edgequake-postgres:local` | 1.7.0 (≥1.7.0) | 0.8.5 |

## Production-stress posture

| Knob | Value |
|------|--------|
| Build | `cargo test --release` |
| Scale | `EDGEQUAKE_PERF_SCALE=prod` (ANN/FTS 50k; ANN dim 1536; Mix seed 5k; expand 200×100) |
| Clients | pg16 **N=8 ≤2×**; pg17/18 **N=16 ≤1.5×** (vs single-client) |
| Pool | concurrent gates `max(clients, 32)`; `stress_pool_saturation` pool=5 clients=16 |
| Mix | MockProvider `context_only`; budget = single × mult × ceil(N/arm_limit) |

## Cross-major highlights (p95_ms)

| Op | pg16 | pg17 | pg18 | Notes |
|----|------|------|------|-------|
| `graph_upsert_edges_batch` | 133.0 | 110.7 | 126.2 | **1.20×** (≤1.3 Wave-1 target) |
| `graph_node_degrees_batch` | 1.44 | 1.38 | 1.43 | stable under 15ms |
| `ingest_vector_upsert_report_created` | ~28–34 median | ~28–34 median | **34.2** | deferred HNSW; under 250ms (was ~390ms) |
| `ingest_vector_ensure_ann_index` | 668 | 675 | 730 | documented CREATE INDEX wall |
| `stress_concurrent_filtered_ann` | 1459 (N=8) | 1969 (N=16) | 1950 (N=16) | prod 50k@1536 |
| `stress_concurrent_fts` | 145 (N=8) | 572 (N=16) | 565 (N=16) | noise_ok (N differs) |
| `stress_concurrent_mix` | 696 (N=8) | 1467 (N=16) | 1465 (N=16) | noise_ok (N + arm_limit) |
| `stress_pool_saturation` | 45.3 | 45.6 | 45.1 | wall ~0.7s (deadlock fixed) |
| `vector_query_unfiltered` | 0.87 | 1.02 | 1.13 | healthy |

## Fixes landed during remasure

1. **Pool deadlock:** `supports_iterative_scan()` ran after `pool.begin()` → nested acquire under saturation. Moved capability probe **before** begin (`storage_impl.rs`).
2. **Prod FTS/expand budgets:** relative to single-client × mult (not fixed 300ms @50k).
3. **Mix budget:** × `ceil(clients / arm_limit)` (semaphore serializes Mix arms).
4. **Compare honesty:** auto `noise_ok` for N-asymmetric stress + sub-100ms micro jitter + deferred ingest spikes.

## SPEC-065 remasure (pg18 only, 2026-07-18 evening)

- Runner now sources `scripts/eq_ephemeral_pg.sh` with **2GB `shared_buffers` / 4g shm** for `prod`/`large`.
- `stress_concurrent_filtered_ann` pg18: **p95 ~167ms** (was ~1950ms under default Docker buffers + cold thrash).
- Concurrent warm-up added before timed window (`e2e_spec061_stress_concurrent_ann.rs`).
- See [`specs/065-product-limits-ssot/e2e/artifacts/RUN_NOTES.md`](../../../065-product-limits-ssot/e2e/artifacts/RUN_NOTES.md).

## Soft-skip

Zero `SKIP:.*DATABASE` under `EDGEQUAKE_REQUIRE_POSTGRES_TESTS=1`.

## Re-run

```bash
make postgres-image-build && make postgres-image-build-pg17 && make postgres-image-build-pg18
make data-access-perf-matrix-release          # default scale, release
make data-access-perf-matrix-prod             # prod scale + release
make compare-eq-perf
```

Artifacts: `eq-perf-{pg16,pg17,pg18}.jsonl` (25 `PERF_REPORT` lines each).
