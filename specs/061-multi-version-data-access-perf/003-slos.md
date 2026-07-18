# SPEC-061 — SLOs and commands

## Single-client SLOs (same on pg16/17/18)

Inherited from SPEC-060 plus Wave 2:

| ID | Target |
|----|--------|
| Q1-c / Q1-d | Filtered HNSW @2k p95&lt;100ms; @50k p95&lt;500ms |
| Q-unfiltered | Unfiltered ANN @10k p95&lt;100ms |
| Q-FTS | FTS @10k p95&lt;200ms |
| Q2-expand | Incident edges p95&lt;100ms |
| Q2-degrees | `node_degrees_batch` 1k p95&lt;100ms |
| I-KV / get / prefix | upsert/get/prefix as in op-matrix |
| I-EDGE | native edge upsert 1k &lt;500ms |
| C-RET | compensate K=1k &lt;500ms |
| Stress | pg16: concurrent p95 ≤ **2×** single-client (N=8); pg17/18: ≤ **1.5×** (N=16) |
| Stress Mix | same mult, × `ceil(N / arm_limit)` (arm semaphore serializes Mix) |
| Stress pool | `stress_pool_saturation`: clients=16, pool=5, p95 &lt;2s (queueing expected) |

### Scale profiles (`EDGEQUAKE_PERF_SCALE`)

| Scale | ANN | FTS | Expand | Mix seed |
|-------|-----|-----|--------|----------|
| `default` (CI) | 10k rows, dim 64 | 10k | 100×50 | 80 @1536 |
| `prod` | 50k rows, dim 1536 | 50k | 200×100 | 5k @1536 |
| `large` (SPEC-063 ladder) | 100k / 500k / 1M via `EDGEQUAKE_CAPACITY_LADDER` | ≤100k | 400×100 | 10k @1536 |

Concurrent gates size the client pool to `max(clients, 32)` except the intentional saturation gate (pool=5).

## Commands

```bash
# Full matrix (builds images if missing, ephemeral containers + sqlx migrate)
make data-access-perf-matrix

# Single profile
EQ_PERF_PROFILES=pg18 make data-access-perf-matrix

# Release build (SPEC-062)
make data-access-perf-matrix-release

# Production-shaped stress + release
make data-access-perf-matrix-prod
# or: EDGEQUAKE_PERF_SCALE=prod EQ_PERF_PROFILES=pg18 make data-access-perf-matrix-prod
```

Requires `sqlx` CLI on PATH (`cargo install sqlx-cli --no-default-features --features postgres --locked`).
The runner applies `edgequake/migrations` on each ephemeral DB (native helpers `eq_next_node_id`).

Artifacts: `/tmp/eq-perf-{pg16,pg17,pg18}.jsonl`  
Archived proof: [`e2e/artifacts/RUN_NOTES.md`](e2e/artifacts/RUN_NOTES.md)

## CI

Nightly [`.github/workflows/postgres-matrix-nightly.yml`](../../.github/workflows/postgres-matrix-nightly.yml) → `spec061-data-access-perf` matrix over `pg16|pg17|pg18` with `EDGEQUAKE_REQUIRE_POSTGRES_TESTS=1`.

PR postgres-integration stays functional-only (no full latency matrix).
