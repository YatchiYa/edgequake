# bench001 LIVE — `smoke`

- **updated:** `2026-07-20T06:21:20Z`
- **started:** `2026-07-20T06:18:20Z`
- **run elapsed:** `3m00s`
- **phase:** `score_parallel` (done)
- **progress:** `10/10`
- **ETA (phase):** `0s`
- **detail:** elapsed=30s

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
| 2026-07-20T06:19:50Z | query_parallel | running | EQ=running LR=done | — | 1m30s |
| 2026-07-20T06:20:01Z | query_eq | running | EQ query 25/40 id=Medical-1991db28 | 1m01s | 1m42s |
| 2026-07-20T06:20:05Z | query_parallel | running | EQ=running LR=done | — | 1m45s |
| 2026-07-20T06:20:11Z | query_eq | running | EQ query 30/40 id=Medical-25f9adbb | 37s | 1m51s |
| 2026-07-20T06:20:20Z | query_parallel | running | EQ=running LR=done | — | 2m00s |
| 2026-07-20T06:20:28Z | query_eq | running | EQ query 35/40 id=Medical-a2771279 | 18s | 2m09s |
| 2026-07-20T06:20:35Z | query_parallel | running | EQ=running LR=done | — | 2m15s |
| 2026-07-20T06:20:41Z | query_eq | running | EQ query 40/40 id=Medical-fa8b9196 | 0s | 2m22s |
| 2026-07-20T06:20:50Z | query_parallel | running | EQ=done LR=done | — | 2m30s |
| 2026-07-20T06:20:50Z | query_parallel | done | eq=40 lr=40 | — | 2m30s |
| 2026-07-20T06:21:20Z | score_parallel | running | EQ=done LR=done eval∥=16 | — | 3m00s |
| 2026-07-20T06:21:20Z | score_parallel | done | elapsed=30s | — | 3m00s |

## Monitor

```bash
make bench001-watch STAGE=smoke
# or:  watch -n 2 cat /Users/raphaelmansuy/Github/03-working/edgequake/specs/001-benchmark/e2e/artifacts/smoke/LIVE.md
# or:  tail -f /Users/raphaelmansuy/Github/03-working/edgequake/specs/001-benchmark/e2e/artifacts/smoke/logs/progress.jsonl
```
