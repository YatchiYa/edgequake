# bench001 LIVE — `smoke`

- **updated:** `2026-07-23T02:38:33Z`
- **started:** `2026-07-23T02:37:33Z`
- **run elapsed:** `1m00s`
- **phase:** `report` (done)
- **progress:** `10/10`
- **ETA (phase):** `0s`
- **detail:** valid=True archive=smoke-20260723T023833Z

## Corpus / chunking

- **docs:** `1`  (done `—`)
- **chunk size / overlap:** `1200` / `100`
- **indexed chunks:** `—`
- **corpus chars:** `1052159`  capped=`False`
- **questions:** `40`

## Pipeline

✓ prepare → ○ ingest_eq → ✓ query_parallel → ✓ score_parallel → ✓ report  |  ● query_lr ● query_eq

## Recent ticks

| at (UTC) | phase | status | detail | eta | run |
|----------|-------|--------|--------|-----|-----|
| 2026-07-23T02:37:48Z | query_parallel | running | EQ=running LR=running | — | 15s |
| 2026-07-23T02:37:49Z | query_lr | running | LR query 25/40 | 9s | 16s |
| 2026-07-23T02:37:51Z | query_eq | running | EQ query 30/40 id=Medical-8f9d5dde | 6s | 18s |
| 2026-07-23T02:37:54Z | query_lr | running | LR query 35/40 | 3s | 21s |
| 2026-07-23T02:37:54Z | query_eq | running | EQ query 35/40 id=Medical-5242d398 | 3s | 22s |
| 2026-07-23T02:37:56Z | query_lr | running | LR query 40/40 | 0s | 24s |
| 2026-07-23T02:37:58Z | query_eq | running | EQ query 40/40 id=Medical-deadc13d | 0s | 25s |
| 2026-07-23T02:38:03Z | query_parallel | running | EQ=done LR=done | — | 30s |
| 2026-07-23T02:38:03Z | query_parallel | done | eq=40 lr=40 | — | 30s |
| 2026-07-23T02:38:33Z | score_parallel | running | EQ=done LR=done eval∥=16 | — | 1m00s |
| 2026-07-23T02:38:33Z | score_parallel | done | elapsed=30s | — | 1m00s |
| 2026-07-23T02:38:33Z | report | done | valid=True archive=smoke-20260723T023833Z | — | 1m00s |

## Monitor

```bash
make bench001-watch STAGE=smoke
# or:  watch -n 2 cat /Users/raphaelmansuy/Github/03-working/edgequake/specs/001-benchmark/e2e/artifacts/smoke/LIVE.md
# or:  tail -f /Users/raphaelmansuy/Github/03-working/edgequake/specs/001-benchmark/e2e/artifacts/smoke/logs/progress.jsonl
```
