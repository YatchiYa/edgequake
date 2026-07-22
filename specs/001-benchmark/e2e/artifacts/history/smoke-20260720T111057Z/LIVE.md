# bench001 LIVE — `smoke`

- **updated:** `2026-07-20T11:10:57Z`
- **started:** `2026-07-20T11:08:57Z`
- **run elapsed:** `2m00s`
- **phase:** `score_parallel` (done)
- **progress:** `10/10`
- **ETA (phase):** `0s`
- **detail:** elapsed=45s

## Corpus / chunking

- **docs:** `1`  (done `—`)
- **chunk size / overlap:** `1200` / `100`
- **indexed chunks:** `—`
- **corpus chars:** `1052159`  capped=`False`
- **questions:** `40`

## Pipeline

✓ prepare → ○ ingest_eq → ✓ query_parallel → ✓ score_parallel → ○ report  |  ● query_lr ● query_eq

## Recent ticks

| at (UTC) | phase | status | detail | eta | run |
|----------|-------|--------|--------|-----|-----|
| 2026-07-20T11:09:26Z | query_eq | running | EQ query 15/40 id=Medical-641dcaf5 | 49s | 29s |
| 2026-07-20T11:09:27Z | query_parallel | running | EQ=running LR=running | — | 30s |
| 2026-07-20T11:09:31Z | query_lr | running | LR query 40/40 | 0s | 34s |
| 2026-07-20T11:09:32Z | query_eq | running | EQ query 20/40 id=Medical-31580ac0 | 35s | 36s |
| 2026-07-20T11:09:42Z | query_parallel | running | EQ=running LR=done | — | 45s |
| 2026-07-20T11:09:50Z | query_eq | running | EQ query 30/40 id=Medical-8f9d5dde | 18s | 53s |
| 2026-07-20T11:09:57Z | query_parallel | running | EQ=running LR=done | — | 1m00s |
| 2026-07-20T11:10:11Z | query_eq | running | EQ query 40/40 id=Medical-c8a65fec | 0s | 1m14s |
| 2026-07-20T11:10:12Z | query_parallel | running | EQ=done LR=done | — | 1m15s |
| 2026-07-20T11:10:12Z | query_parallel | done | eq=40 lr=40 | — | 1m15s |
| 2026-07-20T11:10:57Z | score_parallel | running | EQ=done LR=done eval∥=8 | — | 2m00s |
| 2026-07-20T11:10:57Z | score_parallel | done | elapsed=45s | — | 2m00s |

## Monitor

```bash
make bench001-watch STAGE=smoke
# or:  watch -n 2 cat /Users/raphaelmansuy/Github/03-working/edgequake/specs/001-benchmark/e2e/artifacts/smoke/LIVE.md
# or:  tail -f /Users/raphaelmansuy/Github/03-working/edgequake/specs/001-benchmark/e2e/artifacts/smoke/logs/progress.jsonl
```
