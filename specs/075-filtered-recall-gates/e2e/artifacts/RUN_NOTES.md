# SPEC-075 RUN_NOTES — Filtered recall gate

- Date: 2026-07-18
- Promote metric: **filtered** recall@20 (workspace filter) — never unfiltered-only
- Wave-2 default unchanged; floors unchanged
- Smoke cargo exit: 0
- Cells: 2 · filtered_recall reports: 2

## Cells

| arm | pass | detail |
|-----|------|--------|
| `wave2_partial_iterative` | True | `arm=wave2_partial_iterative rows=2000 filtered_recall=1.0000 single_p95=59.42 slo_ok=true full_green=true (soft-fail product gate)` |
| `iterative_scan_only` | True | `arm=iterative_scan_only rows=2000 filtered_recall=1.0000 single_p95=58.89 slo_ok=true full_green=true (soft-fail product gate)` |

## iterative_scan bounds

- Filtered: `SET LOCAL hnsw.iterative_scan` + `max_scan_tuples` (contract_spec075)
- Unfiltered: iterative_scan **off**
- Env: `EDGEQUAKE_HNSW_ITERATIVE_SCAN`, `EDGEQUAKE_HNSW_MAX_SCAN_TUPLES`, `EDGEQUAKE_HNSW_SCAN_MEM_MULTIPLIER`

## 100k evidence

See [SPEC-068 RUN_NOTES](../../../068-recall-quality-scale/e2e/artifacts/RUN_NOTES.md) — mid-scale wall; Wave-2 `highest_green_N=100000`.
