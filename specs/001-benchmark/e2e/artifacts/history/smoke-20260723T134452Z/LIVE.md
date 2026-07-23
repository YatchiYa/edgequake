# bench001 LIVE — `smoke`

- **updated:** `2026-07-23T13:44:52Z`
- **started:** `2026-07-23T13:42:22Z`
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

✓ prepare → ○ ingest_eq → ✓ query_parallel → ✓ score_parallel → ○ report  |  ● query_eq ● query_lr

## Recent ticks

| at (UTC) | phase | status | detail | eta | run |
|----------|-------|--------|--------|-----|-----|
| 2026-07-23T13:43:17Z | query_lr | running | LR query 25/40 | 32s | 55s |
| 2026-07-23T13:43:20Z | query_eq | running | EQ query 40/40 id=Medical-fa8b9196 | 0s | 58s |
| 2026-07-23T13:43:22Z | query_parallel | running | EQ=done LR=running | — | 1m00s |
| 2026-07-23T13:43:37Z | query_lr | running | LR query 30/40 | 25s | 1m15s |
| 2026-07-23T13:43:52Z | query_parallel | running | EQ=done LR=running | — | 1m30s |
| 2026-07-23T13:44:00Z | query_lr | running | LR query 35/40 | 14s | 1m37s |
| 2026-07-23T13:44:07Z | query_parallel | running | EQ=done LR=running | — | 1m45s |
| 2026-07-23T13:44:12Z | query_lr | running | LR query 40/40 | 0s | 1m50s |
| 2026-07-23T13:44:22Z | query_parallel | running | EQ=done LR=done | — | 2m00s |
| 2026-07-23T13:44:22Z | query_parallel | done | eq=40 lr=40 | — | 2m00s |
| 2026-07-23T13:44:52Z | score_parallel | running | EQ=done LR=done eval∥=24 | — | 2m30s |
| 2026-07-23T13:44:52Z | score_parallel | done | elapsed=30s | — | 2m30s |

## Monitor

```bash
make bench001-watch STAGE=smoke
# or:  watch -n 2 cat /Users/raphaelmansuy/Github/03-working/edgequake/specs/001-benchmark/e2e/artifacts/smoke/LIVE.md
# or:  tail -f /Users/raphaelmansuy/Github/03-working/edgequake/specs/001-benchmark/e2e/artifacts/smoke/logs/progress.jsonl
```
