# bench001 LIVE — `smoke`

- **updated:** `2026-07-20T02:42:33Z`
- **started:** `2026-07-20T02:40:03Z`
- **run elapsed:** `2m30s`
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
| 2026-07-20T02:41:03Z | query_parallel | running | EQ=running LR=done | — | 1m00s |
| 2026-07-20T02:41:10Z | query_eq | running | EQ query 25/40 id=Medical-6809b810 | 40s | 1m07s |
| 2026-07-20T02:41:18Z | query_parallel | running | EQ=running LR=done | — | 1m15s |
| 2026-07-20T02:41:20Z | query_eq | running | EQ query 30/40 id=Medical-25f9adbb | 26s | 1m18s |
| 2026-07-20T02:41:33Z | query_parallel | running | EQ=running LR=done | — | 1m30s |
| 2026-07-20T02:41:45Z | query_eq | running | EQ query 35/40 id=Medical-5242d398 | 15s | 1m43s |
| 2026-07-20T02:41:48Z | query_parallel | running | EQ=running LR=done | — | 1m45s |
| 2026-07-20T02:41:57Z | query_eq | running | EQ query 40/40 id=Medical-fa8b9196 | 0s | 1m54s |
| 2026-07-20T02:42:03Z | query_parallel | running | EQ=done LR=done | — | 2m00s |
| 2026-07-20T02:42:03Z | query_parallel | done | eq=40 lr=40 | — | 2m00s |
| 2026-07-20T02:42:33Z | score_parallel | running | EQ=done LR=done eval∥=24 | — | 2m30s |
| 2026-07-20T02:42:33Z | score_parallel | done | elapsed=30s | — | 2m30s |

## Monitor

```bash
make bench001-watch STAGE=smoke
# or:  watch -n 2 cat /Users/raphaelmansuy/Github/03-working/edgequake/specs/001-benchmark/e2e/artifacts/smoke/LIVE.md
# or:  tail -f /Users/raphaelmansuy/Github/03-working/edgequake/specs/001-benchmark/e2e/artifacts/smoke/logs/progress.jsonl
```
