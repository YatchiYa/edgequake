# SPEC-078 RUN_NOTES — Filtered-DiskANN labels bake-off

- Date: 2026-07-18
- Promote metric: **filtered** recall@20 (labels vs Wave-2 reference)
- Wave-2 remains product default; Filtered-DiskANN labels is **opt-in study**
- Floors unchanged (no raise from smoke)
- Silent flip: forbidden (`EDGEQUAKE_FILTERED_DISKANN_LABELS` default OFF; no product labels migration)
- Contract exit: 0
- Smoke cargo exit: 0
- Cells: 3 · filtered_recall reports: 2

## Cells

| arm | pass | detail |
|-----|------|--------|
| `wave2_filtered` | True | `hits=20` |
| `postfilter_diskann` | True | `hits=20 recall_vs_wave2=1.0000 (post-filter cliff archive)` |
| `filtered_diskann_labels` | True | `wave2_hits=20 labels_hits=20 recall=1.0000 (soft-fail; Wave-2 remains default; no silent flip)` |
| `postfilter_vs_wave2` | True | `FILTERED recall@20 postfilter_vs_wave2=1.0000 (honesty baseline) post_ms=3.0 hits=20` |
| `labels_vs_wave2` | True | `FILTERED recall@20 labels_vs_wave2=1.0000 soft=0.9 wave2_ms=3.4 labels_ms=2.8 rows=2000` |

## Helpers

- `WorkspaceLabelMap` — dense workspace→`smallint` (fail closed at 32767)
- `build_diskann_labels_index_sql` — `USING diskann (embedding …, labels)`
- `build_filtered_diskann_label_select_sql` — `labels && ARRAY[$n]::smallint[]`
- `build_postfilter_diskann_select_sql` — TEXT workspace honesty baseline
- Env: `EDGEQUAKE_FILTERED_DISKANN_LABELS` (default off)

## Decision

Do **not** silent-flip product default from this smoke. Re-run at mid-scale + full gate before any promote.
