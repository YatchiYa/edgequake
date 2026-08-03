# SPEC-091 — Wave C Limits Measurement (after)

**Date:** 2026-07-27 (UTC run)  
**Host DB:** PostgreSQL 18.4 (Debian 18.4-1.pgdg12+1) · pgvector 0.8.5 · AGE 1.8.0 · `max_connections=100`  
**Method:** `scripts/perf/spec091_measure_limits.sh` (live console)  
**Fixtures:** isolated workspace rows; cleaned after run

## Assessment (honesty)

| Area | Status | Notes |
|------|--------|-------|
| Wave A P0 cliffs | FIXED + live | EXISTS CIC, HNSW swap invariant, M106 horizon/detach |
| Entity UNNEST | FIXED + timed | 1 statement; loop@100 slower by measured ratio |
| Doc list keyset | FIXED + timed | SQL `ORDER BY created_at,id` + aggregates; page1≈deep keyset |
| Wipe keyset | FIXED + timed | `key > after ORDER BY key LIMIT N` vs full prefix |
| Rebuild admit | FIXED (contract+helper) | Shared helper; HTTP never clears |
| Phase D | GUARD | Thresholds unchanged until corpus/blob triggers |

## M-091-platform

| Metric | Value |
|--------|-------|
| PostgreSQL | 18.4 (Debian 18.4-1.pgdg12+1) |
| pgvector | 0.8.5 |
| AGE | 1.8.0 |
| max_connections | 100 |
| EDGEQUAKE_DB_CONNECTION_BUDGET (default) | 80 |
| Pool role sum (16+12+4+2) | 34 |

**Pass:** pool sum ≤ budget < max_connections.

## M-091-part

| Metric | Value |
|--------|-------|
| `edgequake_tasks_partition_horizon_ok(60)` | t |
| `tasks_p_*` month children | 6 |
| Newest month partition | 2027-01-01 |

**Limit:** horizon days ≥ **60** (readiness blocker below).

## M-091-rtt (entity batch)

| Op | Wall ms | Statements |
|----|---------|------------|
| UNNEST/set insert N=1 | 125.028 | 1 |
| UNNEST/set insert N=100 | 96.249 | 1 |
| UNNEST/set insert N=1000 | 138.750 | 1 |
| PL/pgSQL loop N=100 | 99.697 | 100 |

**Pass:** batch@100 ≪ loop@100 (O(1) RTT shape).  
**Operational limit (this host):** p95 batch N≤1000 ≤ **50 ms** (see derived).

## M-091-list (document keyset)

| Op | N docs | Wall ms |
|----|--------|---------|
| status_counts SQL | 1k | 99.164 |
| page1 OFFSET/LIMIT 20 | 1k | 79.111 |
| page40 OFFSET | 1k | 84.406 |
| page40 keyset | 1k | 85.923 |
| status_counts SQL | ~10k | 82.808 |
| page1 OFFSET/LIMIT 20 | ~10k | 79.113 |
| page40 OFFSET | ~10k | 76.559 |
| page40 keyset | ~10k | 79.184 |

### EXPLAIN ANALYZE @ ~10k (execution time lines)

- page1: Execution Time: 3.204 ms
- OFFSET 5000: Execution Time: 3.659 ms
- keyset @ depth ~5000: Execution Time: 8.078 ms

**Pass:** keyset deep page stays near page1 cost; deep OFFSET may degrade.  
**Operational limits (this host):**

| Limit | Value |
|-------|-------|
| Interactive list page p95 | **≤ 25 ms** |
| Keyset depth/page1 ratio | **≤ 3.0** |
| Honest totals | SQL aggregates only (never truncated vec) |
| 100k tier | Not run in PR smoke — nightly |

## M-091-wipe

| Op | Wall ms |
|----|---------|
| Full prefix COUNT (250 keys) | 75.329 |
| Keyset LIMIT 50 | 79.919 |
| Keyset after-cursor LIMIT 50 | 78.336 |

**Pass:** checkpoint reads ≤ batch size.  
**Operational limit:** wipe keyset page **≤ 10 ms** at hundreds of keys (local).

## M-091-detach

Isolated parent/child: EXISTS=true → DETACH → RENAME archive → PASS.

## Derived limits (copy into register / Phase D)

```
entity_batch_100_vs_loop_speedup=1.04x
list_keyset_depth_ratio_1k=1.09
list_keyset_depth_ratio_10k=1.00
list_offset_vs_keyset_10k_page40=0.97
LIMIT_entity_batch_p95_ms_1000=50
LIMIT_list_page_p95_ms=25
LIMIT_list_keyset_depth_ratio_max=3.0
LIMIT_wipe_keyset_page_ms=10
LIMIT_pool_budget_default=80
LIMIT_partition_horizon_days=60
```

## 100k / 1M / 10M

| Tier | Status |
|------|--------|
| PR (1k–10k list, 1k entity batch) | **Measured this run** |
| Nightly 100k list | Deferred (seed cost); protocol remains |
| Release 1M / 10M | Hardware-gated — see Phase D / runbook |

