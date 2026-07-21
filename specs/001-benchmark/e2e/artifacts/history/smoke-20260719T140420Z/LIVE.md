# bench001 LIVE — `smoke`

- **updated:** `2026-07-19T14:04:20Z`
- **started:** `2026-07-19T14:02:19Z`
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
| 2026-07-19T14:02:55Z | query_eq | running | EQ query 20/40 id=Medical-7b682af7 | 35s | 35s |
| 2026-07-19T14:02:59Z | query_lr | running | LR query 40/40 | 0s | 39s |
| 2026-07-19T14:03:05Z | query_parallel | running | EQ=running LR=done | — | 45s |
| 2026-07-19T14:03:17Z | query_eq | running | EQ query 30/40 id=Medical-7b694381 | 19s | 57s |
| 2026-07-19T14:03:20Z | query_parallel | running | EQ=running LR=done | — | 1m00s |
| 2026-07-19T14:03:30Z | query_eq | running | EQ query 35/40 id=Medical-d96c57fa | 10s | 1m11s |
| 2026-07-19T14:03:35Z | query_parallel | running | EQ=running LR=done | — | 1m15s |
| 2026-07-19T14:03:41Z | query_eq | running | EQ query 40/40 id=Medical-c8a65fec | 0s | 1m21s |
| 2026-07-19T14:03:50Z | query_parallel | running | EQ=done LR=done | — | 1m30s |
| 2026-07-19T14:03:50Z | query_parallel | done | eq=40 lr=40 | — | 1m30s |
| 2026-07-19T14:04:20Z | score_parallel | running | EQ=done LR=done eval∥=24 | — | 2m00s |
| 2026-07-19T14:04:20Z | score_parallel | done | elapsed=30s | — | 2m00s |

## Monitor

```bash
make bench001-watch STAGE=smoke
# or:  watch -n 2 cat /Users/raphaelmansuy/Github/03-working/edgequake/specs/001-benchmark/e2e/artifacts/smoke/LIVE.md
# or:  tail -f /Users/raphaelmansuy/Github/03-working/edgequake/specs/001-benchmark/e2e/artifacts/smoke/logs/progress.jsonl
```
