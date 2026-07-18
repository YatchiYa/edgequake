# SPEC-070 DiskANN battle summary

- arm summaries: 6
- stress cells: 6
- decision: `green_150k_diskann=false any_diskann_full_green=true diskann_extension_ok=true smoke=false promote_ssot=false (full gate: single∧recall@20≥0.99∧concurrent@clients=16)` pass=False

| detail | pass | p95_ms | plan_class |
|--------|------|--------|------------|
| `rows=100000 full_green=false recall_ok=true abs_ok=false single_p95=162.50 stress_p95=3485.74` | False | 3485.742792 | hnsw_dedicated |
| `rows=100000 full_green=true recall_ok=true abs_ok=true single_p95=1.95 stress_p95=20.30` | True | 20.298917 | diskann_dedicated |
| `rows=150000 full_green=false recall_ok=true abs_ok=false single_p95=213.72 stress_p95=5675.96` | False | 5675.960583 | hnsw_dedicated |
| `rows=150000 full_green=false recall_ok=false abs_ok=true single_p95=2.43 stress_p95=17.08` | False | 17.082709 | diskann_dedicated |
| `rows=250000 full_green=false recall_ok=true abs_ok=false single_p95=299.32 stress_p95=7967.56` | False | 7967.55775 | hnsw_dedicated |
| `rows=250000 full_green=false recall_ok=false abs_ok=true single_p95=2.27 stress_p95=17.29` | False | 17.288208 | diskann_dedicated |
