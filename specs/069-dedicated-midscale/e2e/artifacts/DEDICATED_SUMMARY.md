# SPEC-069 dedicated mid-scale summary

- stress cells: 14
- full_green (clients=16 ladder/contention): 2
- decision: `green_150k=false first_fail=Some((100000, 80)) open_spec070=true` pass=False

| detail | pass | p95_ms |
|--------|------|--------|
| `arm=ladder rows=100000 ef=80 clients=16 single_p95=165.75 abs_ok=false rel_ok=false wall=37.802174875s full_green=false` | False | 3108.8015 |
| `arm=ladder rows=100000 ef=240 clients=16 single_p95=174.58 abs_ok=false rel_ok=false wall=34.374040959s full_green=false` | False | 2813.328709 |
| `arm=ladder rows=125000 ef=80 clients=16 single_p95=207.82 abs_ok=false rel_ok=false wall=44.04602675s full_green=false` | False | 3712.244542 |
| `arm=ladder rows=125000 ef=240 clients=16 single_p95=241.99 abs_ok=false rel_ok=false wall=47.746122125s full_green=false` | False | 3789.201958 |
| `arm=ladder rows=150000 ef=80 clients=16 single_p95=301.37 abs_ok=false rel_ok=false wall=58.414720375s full_green=false` | False | 4730.213250000001 |
| `arm=ladder rows=150000 ef=240 clients=16 single_p95=281.09 abs_ok=false rel_ok=false wall=57.079269625s full_green=false` | False | 4611.528458999999 |
| `arm=ladder rows=200000 ef=80 clients=16 single_p95=274.76 abs_ok=false rel_ok=false wall=74.385045959s full_green=false` | False | 6232.688958 |
| `arm=ladder rows=200000 ef=240 clients=16 single_p95=264.12 abs_ok=false rel_ok=false wall=76.812561s full_green=false` | False | 6089.206708 |
| `arm=contention rows=100000 ef=80 clients=4 single_p95=158.41 abs_ok=true rel_ok=false wall=5.080225375s full_green=true` | True | 279.08375 |
| `arm=contention_scanmem2 rows=100000 ef=80 clients=4 single_p95=157.17 abs_ok=true rel_ok=false wall=6.901855666s full_green=true` | True | 404.48233300000004 |
| `arm=contention rows=100000 ef=80 clients=8 single_p95=165.47 abs_ok=false rel_ok=false wall=19.351876625s full_green=false` | False | 1428.334958 |
| `arm=contention_scanmem2 rows=100000 ef=80 clients=8 single_p95=160.22 abs_ok=false rel_ok=false wall=19.907286042s full_green=false` | False | 1445.2322920000001 |
| `arm=contention rows=100000 ef=80 clients=16 single_p95=173.81 abs_ok=false rel_ok=false wall=38.276799458s full_green=false` | False | 3176.5988749999997 |
| `arm=contention_scanmem2 rows=100000 ef=80 clients=16 single_p95=197.44 abs_ok=false rel_ok=false wall=37.414118917s full_green=false` | False | 3054.542458 |
