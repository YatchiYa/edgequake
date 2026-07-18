# SPEC-075 — First principles (filtered precision)

## Precision law (pgvector official)

With post-filter ANN and `ef_search = E`, expected matching hits ≈ \(E \times selectivity\).  
At 10% workspace selectivity and default `ef_search=40` → ~4 usable rows — not `LIMIT 20`.

**Fixes (industry order):**

1. **Partial HNSW** / partition / dedicated table so the index shape ≡ filter (Wave-2)
2. **`hnsw.iterative_scan`** (`relaxed_order` for most RAG) so the index keeps scanning until enough filtered hits
3. Bound cost with **`hnsw.max_scan_tuples`** (default 20 000) and optional `hnsw.scan_mem_multiplier`
4. Raise `ef_search` (latency cost)

## EdgeQuake defaults

| Query shape | iterative_scan | max_scan_tuples | Wave-2 partial |
|-------------|----------------|-----------------|----------------|
| Filtered (`workspace_id=…`) | `relaxed_order` (unless env `off`) | from `EDGEQUAKE_HNSW_MAX_SCAN_TUPLES` (default 20k) | Opt-in partial index |
| Unfiltered | **not set** (stay off) | n/a | n/a |

Env knobs (SPEC-065 SSOT → [`hnsw_runtime_policy.rs`](../../edgequake/crates/edgequake-storage/src/adapters/postgres/hnsw_runtime_policy.rs)):

- `EDGEQUAKE_HNSW_ITERATIVE_SCAN` — `relaxed_order` (default) / `strict_order` / `off`
- `EDGEQUAKE_HNSW_MAX_SCAN_TUPLES` — bound iterative scan work
- `EDGEQUAKE_HNSW_SCAN_MEM_MULTIPLIER` — raise if list alone does not restore recall
- `EDGEQUAKE_HNSW_EF_SEARCH` — query ef tip (e.g. 240 @100k concurrent)

## Claim honesty

- Promote / gate only with **filtered** recall@20 under a workspace predicate.
- Unfiltered latency demos are not a scale win.
- SPEC-068 archives mid-scale wall @100k Wave-2; this pack’s smoke gate re-asserts the discipline at small N + points at that evidence.
