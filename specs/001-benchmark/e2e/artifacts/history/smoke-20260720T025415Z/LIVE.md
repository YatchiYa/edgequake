# bench001 LIVE — `smoke`

- **updated:** `2026-07-20T02:54:15Z`
- **started:** `2026-07-20T02:52:15Z`
- **run elapsed:** `2m00s`
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
| 2026-07-20T02:52:48Z | query_lr | running | LR query 40/40 | 0s | 32s |
| 2026-07-20T02:52:52Z | query_eq | running | EQ query 15/40 id=Medical-641dcaf5 | 1m00s | 36s |
| 2026-07-20T02:53:00Z | query_parallel | running | EQ=running LR=done | — | 45s |
| 2026-07-20T02:53:09Z | query_eq | running | EQ query 25/40 id=Medical-b5a3c96e | 32s | 54s |
| 2026-07-20T02:53:15Z | query_parallel | running | EQ=running LR=done | — | 1m00s |
| 2026-07-20T02:53:22Z | query_eq | running | EQ query 30/40 id=Medical-8f9d5dde | 22s | 1m06s |
| 2026-07-20T02:53:30Z | query_parallel | running | EQ=running LR=done | — | 1m15s |
| 2026-07-20T02:53:45Z | query_eq | running | EQ query 40/40 id=Medical-fa8b9196 | 0s | 1m29s |
| 2026-07-20T02:53:45Z | query_parallel | running | EQ=done LR=done | — | 1m30s |
| 2026-07-20T02:53:45Z | query_parallel | done | eq=40 lr=40 | — | 1m30s |
| 2026-07-20T02:54:15Z | score_parallel | running | EQ=done LR=done eval∥=24 | — | 2m00s |
| 2026-07-20T02:54:15Z | score_parallel | done | elapsed=30s | — | 2m00s |

## Monitor

```bash
make bench001-watch STAGE=smoke
# or:  watch -n 2 cat /Users/raphaelmansuy/Github/03-working/edgequake/specs/001-benchmark/e2e/artifacts/smoke/LIVE.md
# or:  tail -f /Users/raphaelmansuy/Github/03-working/edgequake/specs/001-benchmark/e2e/artifacts/smoke/logs/progress.jsonl
```
