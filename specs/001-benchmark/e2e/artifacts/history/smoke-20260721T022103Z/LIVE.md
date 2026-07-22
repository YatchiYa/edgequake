# bench001 LIVE — `smoke`

- **updated:** `2026-07-21T02:21:03Z`
- **started:** `2026-07-21T02:18:18Z`
- **run elapsed:** `2m45s`
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

✓ prepare → ○ ingest_eq → ✓ query_parallel → ✓ score_parallel → ○ report  |  ● query_eq ● query_lr

## Recent ticks

| at (UTC) | phase | status | detail | eta | run |
|----------|-------|--------|--------|-----|-----|
| 2026-07-21T02:19:23Z | query_eq | running | EQ query 40/40 id=Medical-fa8b9196 | 0s | 1m05s |
| 2026-07-21T02:19:33Z | query_lr | running | LR query 25/40 | 44s | 1m15s |
| 2026-07-21T02:19:48Z | query_parallel | running | EQ=done LR=running | — | 1m30s |
| 2026-07-21T02:19:49Z | query_lr | running | LR query 30/40 | 30s | 1m31s |
| 2026-07-21T02:20:03Z | query_parallel | running | EQ=done LR=running | — | 1m45s |
| 2026-07-21T02:20:08Z | query_lr | running | LR query 35/40 | 15s | 1m50s |
| 2026-07-21T02:20:18Z | query_parallel | running | EQ=done LR=running | — | 2m00s |
| 2026-07-21T02:20:24Z | query_lr | running | LR query 40/40 | 0s | 2m06s |
| 2026-07-21T02:20:33Z | query_parallel | running | EQ=done LR=done | — | 2m15s |
| 2026-07-21T02:20:33Z | query_parallel | done | eq=40 lr=40 | — | 2m15s |
| 2026-07-21T02:21:03Z | score_parallel | running | EQ=done LR=done eval∥=8 | — | 2m45s |
| 2026-07-21T02:21:03Z | score_parallel | done | elapsed=30s | — | 2m45s |

## Monitor

```bash
make bench001-watch STAGE=smoke
# or:  watch -n 2 cat /Users/raphaelmansuy/Github/03-working/edgequake/specs/001-benchmark/e2e/artifacts/smoke/LIVE.md
# or:  tail -f /Users/raphaelmansuy/Github/03-working/edgequake/specs/001-benchmark/e2e/artifacts/smoke/logs/progress.jsonl
```
