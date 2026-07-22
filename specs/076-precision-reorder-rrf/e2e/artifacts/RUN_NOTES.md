# SPEC-076 RUN_NOTES — Precision layers

- Date: 2026-07-18
- Floors unchanged (Wave-2 100k; DiskANN opt-in 150k)
- Silent flip: forbidden (exact reorder default OFF; sparse fusion default weighted)
- A3 contract exit: 0
- A4 lexical/RRF tip contract exit: 0
- A3 DB smoke: ran exit=0

## A3 — Opt-in exact reorder

- Env: `EDGEQUAKE_ANN_EXACT_REORDER=0|1` (default 0)
- Env: `EDGEQUAKE_ANN_REORDER_CANDIDATE_K` (default 50)
- SQL: MATERIALIZED CTE → `ORDER BY distance + 0` → LIMIT top_k
- Filter columns stay inside the CTE (workspace/tenant)

## A4 — Sparse FTS+ANN RRF tip

- Default: sparse-first weighted (`EDGEQUAKE_SPARSE_FUSION` unset)
- Tip: `EDGEQUAKE_SPARSE_FUSION=rrf` recovers lexical SKU in top-3 vs ANN-only miss
- `content_tsv` upsert honesty asserted (SPEC-058/M091)
- Mix/RRF ≠ promoted ANN floor

## Gate: GREEN

