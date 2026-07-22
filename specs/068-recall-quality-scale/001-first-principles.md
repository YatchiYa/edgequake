# SPEC-068 — First principles

## Problem

SPEC-067 made Wave-2 filtered ANN **latency- and concurrent-green** at 250k (`uses_partial=true`), but **ANN-relative recall@20 ≈ 0.56** vs `ef_search=400`. Floors stay at `highest_green_N=100k`.

## Principles (O(hot-set))

1. **Work tracks hot workspace size**, not global corpus N — partial HNSW + session-local planner bias (SPEC-067) stay mandatory for supported floors.
2. **Filtered ANN quality = candidate depth after filter** — pgvector 0.8 iterative scan (`relaxed_order`) already on; tune `ef_search` / `max_scan_tuples` / `scan_mem_multiplier` before changing index topology.
3. **Build ≠ query** — `ef_construction` / `m` need REINDEX; measure as a separate rebuild arm only if query-time ef cannot hit the recall gate without blowing Q1-d.
4. **Exact top-k is O(N)** — honesty-only; promote on ANN-relative recall@20 ≥ 0.99 (SPEC-064/066).
5. **AGE work is O(entry + hops)** — index entry-point keys; push `DISTINCT`/`LIMIT` into Cypher; keep native SQL expand.
6. **Evidence before claims** — no SSOT raise without SPEC-068 JSONL.

## Non-goals

- Silent product-default flip of halfvec / `ef_search` / `ef_construction`
- DiskANN / pgvectorscale (deferred until hang/FORBIDDEN cliff)
- Raising community Louvain / full-graph scan gates without separate proof

## Promote law

```
rung_green ⇔ single_p95 < 500ms ∧ recall@20_ann ≥ 0.99 ∧ concurrent_abs_p95 < 500ms
```

Latency-only green cells are **not** promoted.
