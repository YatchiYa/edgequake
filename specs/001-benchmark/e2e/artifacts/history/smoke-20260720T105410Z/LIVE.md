# bench001 LIVE — `smoke`

- **updated:** `2026-07-20T10:54:09Z`
- **started:** `2026-07-20T10:51:54Z`
- **run elapsed:** `2m15s`
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
| 2026-07-20T10:52:25Z | query_lr | running | LR query 40/40 | 0s | 31s |
| 2026-07-20T10:52:33Z | query_eq | running | EQ query 20/40 id=Medical-7b682af7 | 39s | 39s |
| 2026-07-20T10:52:39Z | query_parallel | running | EQ=running LR=done | — | 45s |
| 2026-07-20T10:52:51Z | query_eq | running | EQ query 30/40 id=Medical-8f9d5dde | 19s | 57s |
| 2026-07-20T10:52:54Z | query_parallel | running | EQ=running LR=done | — | 1m00s |
| 2026-07-20T10:53:02Z | query_eq | running | EQ query 35/40 id=Medical-d96c57fa | 10s | 1m07s |
| 2026-07-20T10:53:09Z | query_parallel | running | EQ=running LR=done | — | 1m15s |
| 2026-07-20T10:53:11Z | query_eq | running | EQ query 40/40 id=Medical-fa8b9196 | 0s | 1m17s |
| 2026-07-20T10:53:24Z | query_parallel | running | EQ=done LR=done | — | 1m30s |
| 2026-07-20T10:53:24Z | query_parallel | done | eq=40 lr=40 | — | 1m30s |
| 2026-07-20T10:54:09Z | score_parallel | running | EQ=done LR=done eval∥=8 | — | 2m15s |
| 2026-07-20T10:54:09Z | score_parallel | done | elapsed=45s | — | 2m15s |

## Monitor

```bash
make bench001-watch STAGE=smoke
# or:  watch -n 2 cat /Users/raphaelmansuy/Github/03-working/edgequake/specs/001-benchmark/e2e/artifacts/smoke/LIVE.md
# or:  tail -f /Users/raphaelmansuy/Github/03-working/edgequake/specs/001-benchmark/e2e/artifacts/smoke/logs/progress.jsonl
```
