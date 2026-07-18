# SPEC-082 RUN_NOTES — Push-scale floors

- Date: 2026-07-18
- Promote metric: filtered recall (A6/Wave-2); DiskANN dedicated full-gate
- Silent flip: forbidden
- Cargo aggregate exit: 0
- **Decision: DiskANN opt-in floor raised to 250k; Wave-2 default unchanged; A6 tip not default; silent flip forbidden**
- DiskANN: DiskANN opt-in highest_green_N→250000 (full-gate @250k) (highest_green_N candidate=250000)
- A6 labels soft-pass=False: FILTERED recall@20 labels_vs_wave2=0.9500 soft=0.9 wave2_ms=24.9 labels_ms=3.9 rows=150000; FILTERED recall@20 labels_vs_wave2=0.0500 soft=0.9 wave2_ms=15.0 labels_ms=4.3 rows=250000
- Wave-2 @150k spot cells: 4 (default floor stays 100k unless separate full-gate)

## Cells

| op | plan_class | pass | detail |
|----|------------|------|--------|
| `fdl078_cell` | `wave2_filtered` | True | `hits=20` |
| `fdl078_filtered_recall` | `postfilter_vs_wave2` | True | `FILTERED recall@20 postfilter_vs_wave2=0.4000 (honesty baseline) post_ms=4.2 hits=20` |
| `fdl078_cell` | `postfilter_diskann` | True | `hits=20 recall_vs_wave2=0.4000 (post-filter cliff archive)` |
| `fdl078_filtered_recall` | `labels_vs_wave2` | True | `FILTERED recall@20 labels_vs_wave2=0.9500 soft=0.9 wave2_ms=24.9 labels_ms=3.9 rows=150000` |
| `fdl078_cell` | `filtered_diskann_labels` | True | `wave2_hits=20 labels_hits=20 recall=0.9500 (soft-fail; Wave-2 remains default; no silent flip)` |
| `fdl078_decision` | `honesty` | True | `Filtered-DiskANN labels is opt-in study; Wave-2 default + floors unchanged; EDGEQUAKE_FILTERED_DISKANN_LABELS ` |
| `fdl078_cell` | `wave2_filtered` | True | `hits=20` |
| `fdl078_filtered_recall` | `postfilter_vs_wave2` | True | `FILTERED recall@20 postfilter_vs_wave2=0.2500 (honesty baseline) post_ms=5.0 hits=20` |
| `fdl078_cell` | `postfilter_diskann` | True | `hits=20 recall_vs_wave2=0.2500 (post-filter cliff archive)` |
| `fdl078_filtered_recall` | `labels_vs_wave2` | False | `FILTERED recall@20 labels_vs_wave2=0.0500 soft=0.9 wave2_ms=15.0 labels_ms=4.3 rows=250000` |
| `fdl078_cell` | `filtered_diskann_labels` | False | `wave2_hits=20 labels_hits=20 recall=0.0500 (soft-fail; Wave-2 remains default; no silent flip)` |
| `fdl078_decision` | `honesty` | True | `Filtered-DiskANN labels is opt-in study; Wave-2 default + floors unchanged; EDGEQUAKE_FILTERED_DISKANN_LABELS ` |
| `fr075_filtered_recall` | `wave2_partial_iterative` | True | `FILTERED workspace_id=ws-a rows=150000 ef=80 vs ref_ef=400 recall@20=1.0000 gate=0.99 partial=true` |
| `fr075_cell` | `wave2_partial_iterative` | True | `arm=wave2_partial_iterative rows=150000 filtered_recall=1.0000 single_p95=128.73 slo_ok=true full_green=true (` |
| `fr075_filtered_recall` | `iterative_scan_only` | True | `FILTERED workspace_id=ws-a rows=150000 ef=80 vs ref_ef=400 recall@20=1.0000 gate=0.99 partial=false` |
| `fr075_cell` | `iterative_scan_only` | False | `arm=iterative_scan_only rows=150000 filtered_recall=1.0000 single_p95=3634.93 slo_ok=false full_green=false (s` |
| `fr075_decision` | `honesty` | True | `filtered recall@20 is the promote metric; Wave-2 default unchanged; 100k evidence: specs/068-recall-quality-sc` |
| `pareto_cell` | `diskann_q400_r200_default_sbq` | False | `rows=250000 build=default_sbq q_list=400 q_rescore=200 recall=0.9600 single_p95=7.48 stress_p95=39.42 full_gre` |
| `pareto_cell` | `diskann_q800_r400_default_sbq` | False | `rows=250000 build=default_sbq q_list=800 q_rescore=400 recall=0.9800 single_p95=18.44 stress_p95=26.66 full_gr` |
| `pareto_cell` | `diskann_q400_r200_hq_n64_s200` | False | `rows=250000 build=hq_n64_s200 q_list=400 q_rescore=200 recall=0.9500 single_p95=9.10 stress_p95=27.55 full_gre` |
| `pareto_cell` | `diskann_q800_r400_hq_n64_s200` | True | `rows=250000 build=hq_n64_s200 q_list=800 q_rescore=400 recall=1.0000 single_p95=10.35 stress_p95=20.16 full_gr` |
| `pareto_rebuild` | `hq_n64_s200` | True | `rebuild_full_green=true rows=250000` |
| `pareto_cell` | `diskann_q400_r200_default_sbq` | True | `rows=100000 build=default_sbq q_list=400 q_rescore=200 recall=1.0000 single_p95=7.22 stress_p95=24.04 full_gre` |
| `pareto_cell` | `diskann_q800_r400_default_sbq` | True | `rows=100000 build=default_sbq q_list=800 q_rescore=400 recall=1.0000 single_p95=9.06 stress_p95=21.53 full_gre` |
| `pareto_spot` | `spot` | True | `rows=100000 any_full_green=true` |
| `pareto_decision` | `promote` | True | `green_150k=false green_250k=true highest_green_N=250000 promote_ssot=true best=build=hq_n64_s200 query_grid_gr` |

- SSOT action: **applied** — DiskANN opt-in `highest_green_N=250000` in `docs/product-limits.md` (PROMOTE_DISKANN_250K present)
- Recipe tip @250k: HQ build (`n=64`, `sls=200`) + `query_search_list_size=800` + `query_rescore=400`
- A6 cliff: soft-green @150k (0.95) → soft-fail @250k (0.05) — tip remains OFF
- Wave-2: single filtered spot @150k full_green — **not** a concurrent floor raise

