# bench001 LIVE — `smoke`

- **updated:** `2026-07-19T15:11:25Z`
- **started:** `2026-07-19T15:08:55Z`
- **run elapsed:** `2m30s`
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
| 2026-07-19T15:09:47Z | query_lr | running | LR query 30/40 | 17s | 53s |
| 2026-07-19T15:09:54Z | query_eq | running | EQ query 25/40 id=Medical-6809b810 | 36s | 59s |
| 2026-07-19T15:09:55Z | query_parallel | running | EQ=running LR=running | — | 1m00s |
| 2026-07-19T15:10:02Z | query_lr | running | LR query 40/40 | 0s | 1m08s |
| 2026-07-19T15:10:10Z | query_parallel | running | EQ=running LR=done | — | 1m15s |
| 2026-07-19T15:10:22Z | query_eq | running | EQ query 35/40 id=Medical-d96c57fa | 13s | 1m28s |
| 2026-07-19T15:10:25Z | query_parallel | running | EQ=running LR=done | — | 1m30s |
| 2026-07-19T15:10:34Z | query_eq | running | EQ query 40/40 id=Medical-fa8b9196 | 0s | 1m39s |
| 2026-07-19T15:10:40Z | query_parallel | running | EQ=done LR=done | — | 1m45s |
| 2026-07-19T15:10:40Z | query_parallel | done | eq=40 lr=40 | — | 1m45s |
| 2026-07-19T15:11:25Z | score_parallel | running | EQ=done LR=done eval∥=24 | — | 2m30s |
| 2026-07-19T15:11:25Z | score_parallel | done | elapsed=45s | — | 2m30s |

## Monitor

```bash
make bench001-watch STAGE=smoke
# or:  watch -n 2 cat /Users/raphaelmansuy/Github/03-working/edgequake/specs/001-benchmark/e2e/artifacts/smoke/LIVE.md
# or:  tail -f /Users/raphaelmansuy/Github/03-working/edgequake/specs/001-benchmark/e2e/artifacts/smoke/logs/progress.jsonl
```
