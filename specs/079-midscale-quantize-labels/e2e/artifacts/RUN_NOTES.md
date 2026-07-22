# SPEC-079 RUN_NOTES — Mid-scale B2 + A6

- Date: 2026-07-18
- Promote metric: **filtered** recall@20 vs Wave-2
- Wave-2 remains product default; B2/A6 remain opt-in study (default OFF)
- Floors unchanged unless explicit full-gate promote (not this pack)
- Silent flip: forbidden
- Cargo aggregate exit: 0
- Filtered recall reports: 6 (pass=4 fail=2)
- **Decision: Not promoted**
- Detail: tip remains study-only; Wave-2 default; no silent flip

## Cells

| op | plan_class | pass | detail |
|----|------------|------|--------|
| `bq077_filtered_recall` | `binary_vs_wave2` | False | `FILTERED recall@20 binary_vs_wave2=0.0000 soft=0.9 wave2_ms=10.1 bq_ms=1.9 candidate_k=200 rows=50000` |
| `bq077_cell` | `binary_rerank` | False | `wave2_hits=20 bq_hits=20 recall=0.0000 (soft-fail; Wave-2 remains default)` |
| `bq077_decision` | `honesty` | True | `binary+rerank is opt-in study; Wave-2 default + floors unchanged; promote only after full gate (not this smoke)` |
| `fdl078_cell` | `wave2_filtered` | True | `hits=20` |
| `fdl078_filtered_recall` | `postfilter_vs_wave2` | True | `FILTERED recall@20 postfilter_vs_wave2=0.5000 (honesty baseline) post_ms=3.6 hits=20` |
| `fdl078_cell` | `postfilter_diskann` | True | `hits=20 recall_vs_wave2=0.5000 (post-filter cliff archive)` |
| `fdl078_filtered_recall` | `labels_vs_wave2` | True | `FILTERED recall@20 labels_vs_wave2=1.0000 soft=0.9 wave2_ms=14.7 labels_ms=3.8 rows=50000` |
| `fdl078_cell` | `filtered_diskann_labels` | True | `wave2_hits=20 labels_hits=20 recall=1.0000 (soft-fail; Wave-2 remains default; no silent flip)` |
| `fdl078_decision` | `honesty` | True | `Filtered-DiskANN labels is opt-in study; Wave-2 default + floors unchanged; EDGEQUAKE_FILTERED_DISKANN_LABELS default OF` |
| `bq077_filtered_recall` | `binary_vs_wave2` | False | `FILTERED recall@20 binary_vs_wave2=0.0000 soft=0.9 wave2_ms=22.1 bq_ms=1.8 candidate_k=200 rows=100000` |
| `bq077_cell` | `binary_rerank` | False | `wave2_hits=20 bq_hits=20 recall=0.0000 (soft-fail; Wave-2 remains default)` |
| `bq077_decision` | `honesty` | True | `binary+rerank is opt-in study; Wave-2 default + floors unchanged; promote only after full gate (not this smoke)` |
| `fdl078_cell` | `wave2_filtered` | True | `hits=20` |
| `fdl078_filtered_recall` | `postfilter_vs_wave2` | True | `FILTERED recall@20 postfilter_vs_wave2=0.5000 (honesty baseline) post_ms=3.6 hits=20` |
| `fdl078_cell` | `postfilter_diskann` | True | `hits=20 recall_vs_wave2=0.5000 (post-filter cliff archive)` |
| `fdl078_filtered_recall` | `labels_vs_wave2` | True | `FILTERED recall@20 labels_vs_wave2=0.9500 soft=0.9 wave2_ms=22.8 labels_ms=7.2 rows=100000` |
| `fdl078_cell` | `filtered_diskann_labels` | True | `wave2_hits=20 labels_hits=20 recall=0.9500 (soft-fail; Wave-2 remains default; no silent flip)` |
| `fdl078_decision` | `honesty` | True | `Filtered-DiskANN labels is opt-in study; Wave-2 default + floors unchanged; EDGEQUAKE_FILTERED_DISKANN_LABELS default OF` |
