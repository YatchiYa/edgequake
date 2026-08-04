# SPEC-108 M vs U simulation (mock-extract)

> Illustrates LAW-X1: document card stores M; graph stores U.

| label | N | yield | reuse | M | U | M/U |
|-------|--:|------:|------:|--:|--:|----:|
| N=8 y=25 reuse=0.4 | 8 | 25 | 0.4 | 200 | 121 | 1.65 |
| N=12 y=25 reuse=0.4 | 12 | 25 | 0.4 | 300 | 181 | 1.66 |
| N=159 y=30 reuse=0.55 | 159 | 30 | 0.55 | 4770 | 2068 | 2.31 |
| N=317 y=30 reuse=0.55 | 317 | 30 | 0.55 | 9510 | 4122 | 2.31 |
| N=309 y=40 reuse=0.7 | 309 | 40 | 0.7 | 12360 | 3709 | 3.33 |

## Partner read

Envelope row M=12360 ≈ partner 12367; U=3709 (M/U=3.33). UI showing M without U looks like “12k entities”.
