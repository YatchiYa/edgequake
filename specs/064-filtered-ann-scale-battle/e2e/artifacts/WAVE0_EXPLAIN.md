# WAVE0 EXPLAIN — SPEC-064 (pg18)

See also locked hypothesis in prior WAVE0 / RUN_NOTES (btree filter → exact scan).

## Effective GUCs (code defaults unless overridden)

- `hnsw.ef_search` ≈ `clamp(4×top_k, 40, 1000)` (top_k=20 → 80)
- `hnsw.iterative_scan = relaxed_order` when filtered + pgvector ≥ 0.8
- `hnsw.max_scan_tuples = 20000`
- storage mode for this arm: **full** (`vector`)

## Baseline single (full_default)

- p95_ms: **1577.046**
- pass (Q1-d): `False`
- detail: `rows=100000 dim=1536 pool=32 q1d_slo_ms=500 slo_pass=false storage=full index=global`

## EXPLAIN (ANALYZE, BUFFERS)

```
Limit  (cost=2829.25..2829.30 rows=20 width=31) (actual time=162.254..162.256 rows=20.00 loops=1)
  Buffers: shared hit=139183 read=22996 written=22853
  ->  Sort  (cost=2829.25..2829.51 rows=101 width=31) (actual time=162.253..162.254 rows=20.00 loops=1)
        Sort Key: ((embedding <=> '[embedding…]'::vector))
        Sort Method: top-N heapsort  Memory: 26kB
        Buffers: shared hit=139183 read=22996 written=22853
        ->  Bitmap Heap Scan on eq_eq_battle064_full_430a629a_vectors  (cost=291.15..2826.57 rows=101 width=31) (actual time=0.552..161.227 rows=20000.00 loops=1)
              Filter: ((metadata ->> 'type'::text) = 'chunk'::text)
              Buffers: shared hit=139180 read=22996 written=22853
              ->  Bitmap Index Scan on eq_eq_battle064_full_430a629a_vectors_tenant_ws_idx  (cost=0.00..291.12 rows=20283 width=0) (actual time=0.305..0.305 rows=20000.00 loops=1)
                    Index Cond: ((tenant_id = 't-battle064'::text) AND (workspace_id = 'ws-a'::text))
                    Index Searches: 1
                    Buffers: shared read=18
  Buffers: shared hit=65 read=12 dirtied=1
Planning Time: 0.394 ms
Execution Time: 162.277 ms
```

Artifacts: `eq-battle-pg18.jsonl`, `RUN_NOTES.md`.

