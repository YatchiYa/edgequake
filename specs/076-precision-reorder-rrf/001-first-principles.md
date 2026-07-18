# SPEC-076 — First principles (precision layers)

## A3 — ANN → exact reorder

HNSW / DiskANN return an **approximate** neighbor order. Industry pattern (pgvector binary-quantize + README iterative_scan):

1. Pull a wider candidate set (`LIMIT candidate_k`, e.g. 50)
2. Re-rank those rows by **exact** stored distance (`ORDER BY distance + 0` on a `MATERIALIZED` CTE)
3. Return `LIMIT top_k` (e.g. 20)

EdgeQuake keeps this **opt-in** (`EDGEQUAKE_ANN_EXACT_REORDER=1`). Default OFF preserves Wave-2 latency shape. Reorder uses the stored embedding type (`vector` / `halfvec`) — not a silent cast to a missing full-precision column.

Env:

- `EDGEQUAKE_ANN_EXACT_REORDER` — `0`/`off` (default) · `1`/`true`/`on`
- `EDGEQUAKE_ANN_REORDER_CANDIDATE_K` — default `50` (clamped ≥ `top_k`)

## A4 — FTS + ANN RRF for codes / names

Embeddings miss exact tokens (part numbers, proper nouns). Postgres FTS (`content_tsv` + `ts_rank_cd`) is free on the same row. Fusion:

| Mode | Env | Default? |
|------|-----|----------|
| Sparse-first weighted | unset / `weighted` | **Yes** |
| RRF tip | `EDGEQUAKE_SPARSE_FUSION=rrf` | Opt-in tip |

`content_tsv` must stay written on vector upsert (SPEC-058 / M091) or FTS underfills.

## Honesty

- Does **not** raise Wave-2 / DiskANN floors.
- Does **not** silent-flip halfvec, partial HNSW, Mix, or Hybrid defaults.
- Mix/RRF seed scales ≪ ANN ladder — never cite as a 100k/150k ANN claim.
