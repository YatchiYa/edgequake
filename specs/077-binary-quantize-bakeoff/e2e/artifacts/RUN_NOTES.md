# SPEC-077 RUN_NOTES — Binary quantize bake-off

- Date: 2026-07-18
- Promote metric: **filtered** recall@20 (binary vs Wave-2 reference)
- Wave-2 remains product default; binary+rerank is **opt-in study**
- Floors unchanged (no raise from smoke)
- Silent flip: forbidden (`EDGEQUAKE_BINARY_QUANTIZE` default OFF)
- Contract exit: 0
- Smoke cargo exit: 0
- Cells: 1 · filtered_recall reports: 1

## Cells

| arm | pass | detail |
|-----|------|--------|
| `binary_rerank` | True | `wave2_hits=20 bq_hits=20 recall=1.0000 (soft-fail; Wave-2 remains default)` |
| `binary_vs_wave2` | True | `FILTERED recall@20 binary_vs_wave2=1.0000 soft=0.9 wave2_ms=3.4 bq_ms=1.2 candidate_k=200 rows=2000` |

## Helpers

- `build_binary_hnsw_index_sql` — expression HNSW `bit_hamming_ops`
- `build_binary_rerank_select_sql` — Hamming candidates → exact halfvec reorder
- Env: `EDGEQUAKE_BINARY_QUANTIZE` (default off), `EDGEQUAKE_BINARY_CANDIDATE_K` (default 200)

## Decision

Do **not** silent-flip product default from this smoke. Re-run at mid-scale + full gate before any promote.
