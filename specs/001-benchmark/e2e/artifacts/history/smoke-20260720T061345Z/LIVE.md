# bench001 LIVE — `smoke`

- **updated:** `2026-07-20T06:13:45Z`
- **started:** `2026-07-20T06:10:45Z`
- **run elapsed:** `3m00s`
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
| 2026-07-20T06:12:00Z | query_parallel | running | EQ=running LR=done | — | 1m15s |
| 2026-07-20T06:12:10Z | query_eq | running | EQ query 25/40 id=Medical-1991db28 | 51s | 1m25s |
| 2026-07-20T06:12:15Z | query_parallel | running | EQ=running LR=done | — | 1m30s |
| 2026-07-20T06:12:25Z | query_eq | running | EQ query 30/40 id=Medical-8f9d5dde | 34s | 1m41s |
| 2026-07-20T06:12:30Z | query_parallel | running | EQ=running LR=done | — | 1m45s |
| 2026-07-20T06:12:40Z | query_eq | running | EQ query 35/40 id=Medical-4654807f | 16s | 1m55s |
| 2026-07-20T06:12:45Z | query_parallel | running | EQ=running LR=done | — | 2m00s |
| 2026-07-20T06:12:52Z | query_eq | running | EQ query 40/40 id=Medical-c8a65fec | 0s | 2m08s |
| 2026-07-20T06:13:00Z | query_parallel | running | EQ=done LR=done | — | 2m15s |
| 2026-07-20T06:13:00Z | query_parallel | done | eq=40 lr=40 | — | 2m15s |
| 2026-07-20T06:13:45Z | score_parallel | running | EQ=done LR=done eval∥=16 | — | 3m00s |
| 2026-07-20T06:13:45Z | score_parallel | done | elapsed=45s | — | 3m00s |

## Monitor

```bash
make bench001-watch STAGE=smoke
# or:  watch -n 2 cat /Users/raphaelmansuy/Github/03-working/edgequake/specs/001-benchmark/e2e/artifacts/smoke/LIVE.md
# or:  tail -f /Users/raphaelmansuy/Github/03-working/edgequake/specs/001-benchmark/e2e/artifacts/smoke/logs/progress.jsonl
```
