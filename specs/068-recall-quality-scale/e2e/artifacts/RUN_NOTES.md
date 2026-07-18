# SPEC-068 recall Pareto — RUN_NOTES

**Date:** 2026-07-18  
**Host:** macOS, **128 GB** RAM  
**Profile:** pg18 + **AGE 1.8.0** / pgvector **0.8.5**  
**Shape:** Wave-2 (halfvec + partial HNSW) + SPEC-067 planner bias  
**Artifact:** [`eq-recall-pareto-pg18.jsonl`](eq-recall-pareto-pg18.jsonl) · [`PARETO_SUMMARY.md`](PARETO_SUMMARY.md)

## Stack

| Component | Version |
|-----------|---------|
| PostgreSQL | 18 |
| pgvector | 0.8.5 |
| Apache AGE | **1.8.0** (`PG18/v1.8.0-rc0`) |

## Full-gate green cells only

Gate = single Q1-d ∧ recall@20≥0.99 (vs ef=400) ∧ concurrent absolute &lt;500 ms.

| Arm | rows | ef_search | Result |
|-----|------|-----------|--------|
| query_ef | **100 000** | **240** | **GREEN** |
| query_ef | **100 000** | **400** | **GREEN** |

No cell with `rows > 100000` was full-gate green.

## Cliffs (honest wall)

| rows | Pattern |
|------|---------|
| 100k | ef=80/160: recall OK; concurrent p95 ~518–526 ms (skirts absolute Q1-d). ef≥240 restores concurrent green. |
| 150k | recall@20 = 1.00 at all measured ef; concurrent ~650–675 ms (**FAIL** abs Q1-d) |
| 200k | ef=80: HNSW-fast (~7 ms) but recall 0.93; ef≥160: recall 1.00 but concurrent ~800–890 ms |
| 250k | ef≤240: latency ~3 ms on HNSW, recall ~0.55; ef=400: recall 1.00, concurrent ~956 ms |
| rebuild `m=32` `ef_c=128` @250k | Index build ~9.6 s; still no full-gate green (ef=240 recall 1.00 but concurrent ~1.1 s) |

## Promotion decision

| Field | Value | Notes |
|-------|-------|-------|
| `highest_green_N` | **100 000** (unchanged) | Confirmed; optional ops tip `ef_search=240` for concurrent headroom |
| `first_fail_N` | **250 000** (unchanged) | Mid-scale fails concurrent and/or recall before a promotable rung |
| Silent default flip | **No** | Do not raise product `ef_search` clamp |
| DiskANN | **Out of scope** | No hang/FORBIDDEN cliff |

## Ops tip (not a default)

At supported **100k** Wave-2, if concurrent p95 skirts 500 ms under default ef (~80):

```bash
export EDGEQUAKE_HNSW_EF_SEARCH=240
```

Do **not** use this tip to market &gt;100k — 150k+ remains not promoted.

## Graph G1 on AGE 1.8

| Metric | Value |
|--------|-------|
| nodes | 100 000 |
| degrees p95 | **~11.7 ms** (SLO &lt;100 ms) |
| plan class | `eq_source_id_btree` |
| Artifact | [`eq-ceiling-pg18-G1-0.jsonl`](eq-ceiling-pg18-G1-0.jsonl) |
| Result | **GREEN** — entry-point denorm indexes healthy on AGE 1.8 |

See also [`002-age-entry-point-audit.md`](../../002-age-entry-point-audit.md).
