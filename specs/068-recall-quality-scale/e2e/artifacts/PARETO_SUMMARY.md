# SPEC-068 recall × latency Pareto summary

- stress cells: 19
- full_green cells: 2

| detail | pass | stress_p95_ms |
|--------|------|---------------|
| `arm=query_ef rows=100000 ef=80 clients=16 single_p95=88.80 abs_ok=false rel_ok=false rel_budget=133.20 wall=7.891406208s` | False | 526.129875 |
| `arm=query_ef rows=100000 ef=160 clients=16 single_p95=146.44 abs_ok=false rel_ok=false rel_budget=219.66 wall=8.26001429` | False | 518.483334 |
| `arm=query_ef rows=100000 ef=240 clients=16 single_p95=70.00 abs_ok=true rel_ok=false rel_budget=105.01 wall=7.590792042s` | True | 432.52475000000004 |
| `arm=query_ef rows=100000 ef=400 clients=16 single_p95=70.98 abs_ok=true rel_ok=false rel_budget=106.47 wall=7.949973s fu` | True | 468.996708 |
| `arm=query_ef rows=150000 ef=80 clients=16 single_p95=103.64 abs_ok=false rel_ok=false rel_budget=155.46 wall=11.74612233` | False | 675.451958 |
| `arm=query_ef rows=150000 ef=160 clients=16 single_p95=117.51 abs_ok=false rel_ok=false rel_budget=176.27 wall=12.1534693` | False | 673.4808340000001 |
| `arm=query_ef rows=150000 ef=240 clients=16 single_p95=103.00 abs_ok=false rel_ok=false rel_budget=154.50 wall=11.8160142` | False | 664.704209 |
| `arm=query_ef rows=150000 ef=400 clients=16 single_p95=104.38 abs_ok=false rel_ok=false rel_budget=156.57 wall=11.9187913` | False | 657.8301250000001 |
| `arm=query_ef rows=200000 ef=80 clients=16 single_p95=7.34 abs_ok=true rel_ok=true rel_budget=50.00 wall=169.193167ms ful` | False | 19.721959000000002 |
| `arm=query_ef rows=200000 ef=160 clients=16 single_p95=152.99 abs_ok=false rel_ok=false rel_budget=229.48 wall=14.1306505` | False | 809.94425 |
| `arm=query_ef rows=200000 ef=240 clients=16 single_p95=146.91 abs_ok=false rel_ok=false rel_budget=220.37 wall=15.1562999` | False | 863.879333 |
| `arm=query_ef rows=200000 ef=400 clients=16 single_p95=142.68 abs_ok=false rel_ok=false rel_budget=214.01 wall=16.0350450` | False | 889.1342500000001 |
| `arm=query_ef rows=250000 ef=80 clients=16 single_p95=2.99 abs_ok=true rel_ok=true rel_budget=50.00 wall=196.591ms full_g` | False | 18.079667 |
| `arm=query_ef rows=250000 ef=160 clients=16 single_p95=3.29 abs_ok=true rel_ok=true rel_budget=50.00 wall=169.413208ms fu` | False | 15.2815 |
| `arm=query_ef rows=250000 ef=240 clients=16 single_p95=3.62 abs_ok=true rel_ok=true rel_budget=50.00 wall=136.38925ms ful` | False | 9.661 |
| `arm=query_ef rows=250000 ef=400 clients=16 single_p95=187.96 abs_ok=false rel_ok=false rel_budget=281.94 wall=17.6251164` | False | 955.686625 |
| `arm=rebuild_m32_efc128 rows=250000 ef=80 clients=16 single_p95=3.97 abs_ok=true rel_ok=true rel_budget=50.00 wall=181.15` | False | 23.272166000000002 |
| `arm=rebuild_m32_efc128 rows=250000 ef=160 clients=16 single_p95=3.74 abs_ok=true rel_ok=true rel_budget=50.00 wall=150.9` | False | 10.598792 |
| `arm=rebuild_m32_efc128 rows=250000 ef=240 clients=16 single_p95=192.40 abs_ok=false rel_ok=false rel_budget=288.60 wall=` | False | 1116.8871250000002 |

## Green cells (promote candidates)
- `arm=query_ef rows=100000 ef=240 clients=16 single_p95=70.00 abs_ok=true rel_ok=false rel_budget=105.01 wall=7.590792042s full_green=true`
- `arm=query_ef rows=100000 ef=400 clients=16 single_p95=70.98 abs_ok=true rel_ok=false rel_budget=106.47 wall=7.949973s full_green=true`
