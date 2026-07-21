# Ablation — Q1_occurrence_sort_p0_v1

**Archive:** `smoke-20260720T023313Z` · P0 BM25 + `KG_CHUNK_OCCURRENCE_SORT=1`

| Metric | Q1 | P0 T013551Z | Gate |
|--------|----|-------------|------|
| EQ Acc | 0.736 | 0.744 | CI not worse |
| Fact Acc | 0.646 | 0.715 | +≥0.03 → miss |
| Fact ctx_rel | 0.450 | 0.450 | +≥0.05 → miss |

Verdict: GATE MISSED
