# bench001 LIVE — `smoke`

- **updated:** `2026-07-23T02:20:38Z`
- **started:** `2026-07-23T02:19:37Z`
- **run elapsed:** `1m00s`
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
| 2026-07-23T02:19:52Z | query_eq | running | EQ query 25/40 id=Medical-6809b810 | 9s | 14s |
| 2026-07-23T02:19:53Z | query_parallel | running | EQ=running LR=running | — | 15s |
| 2026-07-23T02:19:54Z | query_lr | running | LR query 25/40 | 9s | 16s |
| 2026-07-23T02:19:56Z | query_eq | running | EQ query 30/40 id=Medical-c2a36052 | 6s | 18s |
| 2026-07-23T02:19:57Z | query_lr | running | LR query 30/40 | 6s | 20s |
| 2026-07-23T02:20:00Z | query_eq | running | EQ query 35/40 id=Medical-4654807f | 3s | 23s |
| 2026-07-23T02:20:04Z | query_lr | running | LR query 40/40 | 0s | 27s |
| 2026-07-23T02:20:05Z | query_eq | running | EQ query 40/40 id=Medical-c8a65fec | 0s | 28s |
| 2026-07-23T02:20:08Z | query_parallel | running | EQ=done LR=done | — | 30s |
| 2026-07-23T02:20:08Z | query_parallel | done | eq=40 lr=40 | — | 30s |
| 2026-07-23T02:20:38Z | score_parallel | running | EQ=done LR=done eval∥=16 | — | 1m00s |
| 2026-07-23T02:20:38Z | score_parallel | done | elapsed=30s | — | 1m00s |

## Monitor

```bash
make bench001-watch STAGE=smoke
# or:  watch -n 2 cat /Users/raphaelmansuy/Github/03-working/edgequake/specs/001-benchmark/e2e/artifacts/smoke/LIVE.md
# or:  tail -f /Users/raphaelmansuy/Github/03-working/edgequake/specs/001-benchmark/e2e/artifacts/smoke/logs/progress.jsonl
```
