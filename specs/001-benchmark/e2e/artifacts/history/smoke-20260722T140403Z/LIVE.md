# bench001 LIVE — `smoke`

- **updated:** `2026-07-22T14:04:03Z`
- **started:** `2026-07-22T14:02:03Z`
- **run elapsed:** `2m00s`
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
| 2026-07-22T14:02:33Z | query_parallel | running | EQ=running LR=running | — | 30s |
| 2026-07-22T14:02:33Z | query_eq | running | EQ query 20/40 id=Medical-31580ac0 | 30s | 30s |
| 2026-07-22T14:02:34Z | query_lr | running | LR query 40/40 | 0s | 32s |
| 2026-07-22T14:02:43Z | query_eq | running | EQ query 25/40 id=Medical-6809b810 | 24s | 41s |
| 2026-07-22T14:02:48Z | query_parallel | running | EQ=running LR=done | — | 45s |
| 2026-07-22T14:02:59Z | query_eq | running | EQ query 35/40 id=Medical-d96c57fa | 8s | 56s |
| 2026-07-22T14:03:03Z | query_parallel | running | EQ=running LR=done | — | 1m00s |
| 2026-07-22T14:03:07Z | query_eq | running | EQ query 40/40 id=Medical-fa8b9196 | 0s | 1m04s |
| 2026-07-22T14:03:18Z | query_parallel | running | EQ=done LR=done | — | 1m15s |
| 2026-07-22T14:03:18Z | query_parallel | done | eq=40 lr=40 | — | 1m15s |
| 2026-07-22T14:04:03Z | score_parallel | running | EQ=done LR=done eval∥=24 | — | 2m00s |
| 2026-07-22T14:04:03Z | score_parallel | done | elapsed=45s | — | 2m00s |

## Monitor

```bash
make bench001-watch STAGE=smoke
# or:  watch -n 2 cat /Users/raphaelmansuy/Github/03-working/edgequake/specs/001-benchmark/e2e/artifacts/smoke/LIVE.md
# or:  tail -f /Users/raphaelmansuy/Github/03-working/edgequake/specs/001-benchmark/e2e/artifacts/smoke/logs/progress.jsonl
```
