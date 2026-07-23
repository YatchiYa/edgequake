# bench001 LIVE — `smoke`

- **updated:** `2026-07-23T13:25:27Z`
- **started:** `2026-07-23T13:13:41Z`
- **run elapsed:** `11m45s`
- **phase:** `score_parallel` (done)
- **progress:** `10/10`
- **ETA (phase):** `0s`
- **detail:** elapsed=1m00s

## Corpus / chunking

- **docs:** `1`  (done `1`)
- **chunk size / overlap:** `1200` / `100`
- **indexed chunks:** `—`
- **corpus chars:** `1052159`  capped=`False`
- **questions:** `40`

## Pipeline

✓ prepare → ✓ ingest_eq → ✓ query_parallel → ✓ score_parallel → ○ report  |  ● query_lr ● query_eq

## Recent ticks

| at (UTC) | phase | status | detail | eta | run |
|----------|-------|--------|--------|-----|-----|
| 2026-07-23T13:23:30Z | ingest_eq | done | wall_s=588.7 | — | 9m49s |
| 2026-07-23T13:23:39Z | query_eq | running | EQ query 10/40 id=Medical-ba2c8007 | 26s | 9m58s |
| 2026-07-23T13:23:41Z | query_parallel | running | EQ=running LR=done | — | 10m00s |
| 2026-07-23T13:23:53Z | query_eq | running | EQ query 25/40 id=Medical-c2a36052 | 14s | 10m12s |
| 2026-07-23T13:23:56Z | query_parallel | running | EQ=running LR=done | — | 10m15s |
| 2026-07-23T13:24:09Z | query_eq | running | EQ query 35/40 id=Medical-a2771279 | 6s | 10m28s |
| 2026-07-23T13:24:11Z | query_parallel | running | EQ=running LR=done | — | 10m30s |
| 2026-07-23T13:24:16Z | query_eq | running | EQ query 40/40 id=Medical-5242d398 | 0s | 10m35s |
| 2026-07-23T13:24:26Z | query_parallel | running | EQ=done LR=done | — | 10m45s |
| 2026-07-23T13:24:26Z | query_parallel | done | eq=40 lr=40 | — | 10m45s |
| 2026-07-23T13:25:27Z | score_parallel | running | EQ=done LR=done eval∥=24 | — | 11m45s |
| 2026-07-23T13:25:27Z | score_parallel | done | elapsed=1m00s | — | 11m45s |

## Monitor

```bash
make bench001-watch STAGE=smoke
# or:  watch -n 2 cat /Users/raphaelmansuy/Github/03-working/edgequake/specs/001-benchmark/e2e/artifacts/smoke/LIVE.md
# or:  tail -f /Users/raphaelmansuy/Github/03-working/edgequake/specs/001-benchmark/e2e/artifacts/smoke/logs/progress.jsonl
```
