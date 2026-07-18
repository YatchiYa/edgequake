# SPEC-064 — Filtered ANN scale battle (close the L1 cliff)

**Status:** Active  
**Depends on:** SPEC-059 (halfvec recall), SPEC-063 (L1 measured cliff ~1.5s @100k/1536)  
**Goal:** Prove which lever closes Q1-d (filtered ANN single p95 **&lt;500ms** @100k dim=1536) without silent recall loss.

## First principles

Cost ≈ **bytes_touched × DIM × iterations**. L1 cliff is the filtered ANN path (workspace filter ~20% of rows) walking a global HNSW via `hnsw.iterative_scan` up to `max_scan_tuples`.

| Rank | Lever | Why | Risk |
|------|-------|-----|------|
| 1 | **halfvec** storage + HNSW | ~0.5× bytes → more index in RAM | Recall (gate ≥0.99 @20) |
| 2 | **Partial HNSW by workspace** | ANN walks a matching subgraph (no over-filter) | DDL / ops for many WS |
| 3 | **GUC grid** | `ef_search`, `max_scan_tuples`, `scan_mem_multiplier` | Blind bump can worsen p95 |

## Pack

| Doc | Content |
|-----|---------|
| [`001-first-principles.md`](001-first-principles.md) | Cost model + filter physics |
| [`002-lever-matrix.md`](002-lever-matrix.md) | Arms, gates, promote rules |
| [`003-battle-harness.md`](003-battle-harness.md) | Env knobs + `make ann-scale-battle` |
| [`e2e/artifacts/`](e2e/artifacts/) | JSONL + WAVE0 EXPLAIN |

## Commands

```bash
# Full battle (pg18 release, 100k @1536) — not PR CI
make ann-scale-battle

# Subset arms
EDGEQUAKE_BATTLE_ARMS=full_default,halfvec_default make ann-scale-battle

# Non-regression floor (50k)
make data-access-perf-matrix-prod
```

## Success gates

| Gate | Target |
|------|--------|
| L1 Q1-d | single p95 **&lt;500ms** @100k/1536 |
| Recall | @20 ≥ **0.99** vs full baseline |
| Stress | N=16 ≤1.5× single on pg18 |
| Honesty | Promote SPEC-063 envelope / FAQ **only** when `battle_gate_summary.pass=true` |

## Related

- Cliff evidence: [`../063-architecture-capacity-assessment/e2e/artifacts/RUN_NOTES.md`](../063-architecture-capacity-assessment/e2e/artifacts/RUN_NOTES.md)
- Data plane: [`../../docs/deep-dives/data-layer.md`](../../docs/deep-dives/data-layer.md)
