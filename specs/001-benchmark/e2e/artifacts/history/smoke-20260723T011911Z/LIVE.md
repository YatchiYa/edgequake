# bench001 LIVE — `smoke`

- **updated:** `2026-07-23T01:19:11Z`
- **started:** `2026-07-23T01:17:41Z`
- **run elapsed:** `1m30s`
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
| 2026-07-23T01:18:01Z | query_lr | running | LR query 35/40 | 3s | 20s |
| 2026-07-23T01:18:01Z | query_eq | running | EQ query 20/40 id=Medical-7b682af7 | 20s | 20s |
| 2026-07-23T01:18:04Z | query_lr | running | LR query 40/40 | 0s | 24s |
| 2026-07-23T01:18:06Z | query_eq | running | EQ query 25/40 id=Medical-6809b810 | 15s | 25s |
| 2026-07-23T01:18:11Z | query_parallel | running | EQ=running LR=done | — | 30s |
| 2026-07-23T01:18:20Z | query_eq | running | EQ query 35/40 id=Medical-4654807f | 6s | 40s |
| 2026-07-23T01:18:26Z | query_parallel | running | EQ=running LR=done | — | 45s |
| 2026-07-23T01:18:27Z | query_eq | running | EQ query 40/40 id=Medical-fa8b9196 | 0s | 46s |
| 2026-07-23T01:18:41Z | query_parallel | running | EQ=done LR=done | — | 1m00s |
| 2026-07-23T01:18:41Z | query_parallel | done | eq=40 lr=40 | — | 1m00s |
| 2026-07-23T01:19:11Z | score_parallel | running | EQ=done LR=done eval∥=24 | — | 1m30s |
| 2026-07-23T01:19:11Z | score_parallel | done | elapsed=30s | — | 1m30s |

## Monitor

```bash
make bench001-watch STAGE=smoke
# or:  watch -n 2 cat /Users/raphaelmansuy/Github/03-working/edgequake/specs/001-benchmark/e2e/artifacts/smoke/LIVE.md
# or:  tail -f /Users/raphaelmansuy/Github/03-working/edgequake/specs/001-benchmark/e2e/artifacts/smoke/logs/progress.jsonl
```
