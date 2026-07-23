# bench001 LIVE — `smoke`

- **updated:** `2026-07-22T13:14:49Z`
- **started:** `2026-07-22T13:13:04Z`
- **run elapsed:** `1m45s`
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
| 2026-07-22T13:13:34Z | query_parallel | running | EQ=running LR=running | — | 30s |
| 2026-07-22T13:13:34Z | query_eq | running | EQ query 20/40 id=Medical-7b682af7 | 30s | 31s |
| 2026-07-22T13:13:41Z | query_lr | running | LR query 40/40 | 0s | 37s |
| 2026-07-22T13:13:43Z | query_eq | running | EQ query 25/40 id=Medical-c2a36052 | 23s | 39s |
| 2026-07-22T13:13:49Z | query_parallel | running | EQ=running LR=done | — | 45s |
| 2026-07-22T13:14:00Z | query_eq | running | EQ query 35/40 id=Medical-d96c57fa | 8s | 56s |
| 2026-07-22T13:14:04Z | query_parallel | running | EQ=running LR=done | — | 1m00s |
| 2026-07-22T13:14:11Z | query_eq | running | EQ query 40/40 id=Medical-fa8b9196 | 0s | 1m08s |
| 2026-07-22T13:14:19Z | query_parallel | running | EQ=done LR=done | — | 1m15s |
| 2026-07-22T13:14:19Z | query_parallel | done | eq=40 lr=40 | — | 1m15s |
| 2026-07-22T13:14:49Z | score_parallel | running | EQ=done LR=done eval∥=24 | — | 1m45s |
| 2026-07-22T13:14:49Z | score_parallel | done | elapsed=30s | — | 1m45s |

## Monitor

```bash
make bench001-watch STAGE=smoke
# or:  watch -n 2 cat /Users/raphaelmansuy/Github/03-working/edgequake/specs/001-benchmark/e2e/artifacts/smoke/LIVE.md
# or:  tail -f /Users/raphaelmansuy/Github/03-working/edgequake/specs/001-benchmark/e2e/artifacts/smoke/logs/progress.jsonl
```
