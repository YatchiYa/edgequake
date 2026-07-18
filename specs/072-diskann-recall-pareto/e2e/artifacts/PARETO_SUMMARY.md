# SPEC-072 DiskANN recall Pareto summary

- cells: 8
- full_green cells: 5
- decision: `green_150k=true promote_ssot=true best=build=default_sbq query_grid_green ref_search_list=1600 smoke=false (full gate: single∧recall@20≥0.99∧concurrent@clients=16) ref_method=high_diskann_query_search_list_size` pass=True

| detail | pass | p95_ms |
|--------|------|--------|
| `rows=150000 build=default_sbq q_list=100 q_rescore=50 recall=0.6500 single_p95=3.83 stress_p95=17.40 full_green=false` | False | 17.401083 |
| `rows=150000 build=default_sbq q_list=200 q_rescore=100 recall=0.9700 single_p95=5.05 stress_p95=12.91 full_green=false` | False | 12.906959 |
| `rows=150000 build=default_sbq q_list=400 q_rescore=200 recall=1.0000 single_p95=6.20 stress_p95=13.03 full_green=true` | True | 13.026042 |
| `rows=150000 build=default_sbq q_list=800 q_rescore=400 recall=1.0000 single_p95=9.03 stress_p95=16.38 full_green=true` | True | 16.380792000000003 |
| `rows=100000 build=default_sbq q_list=400 q_rescore=200 recall=1.0000 single_p95=5.04 stress_p95=20.01 full_green=true` | True | 20.011917 |
| `rows=100000 build=default_sbq q_list=800 q_rescore=400 recall=1.0000 single_p95=7.61 stress_p95=15.69 full_green=true` | True | 15.691291999999999 |
| `rows=250000 build=default_sbq q_list=400 q_rescore=200 recall=0.9400 single_p95=6.84 stress_p95=21.87 full_green=false` | False | 21.871417 |
| `rows=250000 build=default_sbq q_list=800 q_rescore=400 recall=1.0000 single_p95=9.44 stress_p95=18.97 full_green=true` | True | 18.966709 |
