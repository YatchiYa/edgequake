# bench001 LIVE — `smoke`

- **updated:** `2026-07-20T16:16:13Z`
- **started:** `2026-07-20T16:02:12Z`
- **run elapsed:** `14m00s`
- **phase:** `score_parallel` (done)
- **progress:** `10/10`
- **ETA (phase):** `0s`
- **detail:** elapsed=45s

## Corpus / chunking

- **docs:** `1`  (done `1`)
- **chunk size / overlap:** `1200` / `100`
- **indexed chunks:** `—`
- **corpus chars:** `1052159`  capped=`False`
- **questions:** `40`

## Pipeline

✓ prepare → ✓ ingest_eq → ✓ query_parallel → ✓ score_parallel → ○ report  |  ● query_lr ● query_eq

## Recent ticks

| at (UTC) | phase | status | detail | eta | run |
|----------|-------|--------|--------|-----|-----|
| 2026-07-20T16:14:27Z | query_parallel | running | EQ=running LR=done | — | 12m15s |
| 2026-07-20T16:14:40Z | query_eq | running | EQ query 20/40 id=Medical-31580ac0 | 24s | 12m27s |
| 2026-07-20T16:14:42Z | query_parallel | running | EQ=running LR=done | — | 12m30s |
| 2026-07-20T16:14:55Z | query_eq | running | EQ query 30/40 id=Medical-7b694381 | 13s | 12m43s |
| 2026-07-20T16:14:57Z | query_parallel | running | EQ=running LR=done | — | 12m45s |
| 2026-07-20T16:15:03Z | query_eq | running | EQ query 35/40 id=Medical-d96c57fa | 7s | 12m51s |
| 2026-07-20T16:15:12Z | query_parallel | running | EQ=running LR=done | — | 13m00s |
| 2026-07-20T16:15:15Z | query_eq | running | EQ query 40/40 id=Medical-fa8b9196 | 0s | 13m03s |
| 2026-07-20T16:15:27Z | query_parallel | running | EQ=done LR=done | — | 13m15s |
| 2026-07-20T16:15:27Z | query_parallel | done | eq=40 lr=40 | — | 13m15s |
| 2026-07-20T16:16:13Z | score_parallel | running | EQ=done LR=done eval∥=8 | — | 14m00s |
| 2026-07-20T16:16:13Z | score_parallel | done | elapsed=45s | — | 14m00s |

## Monitor

```bash
make bench001-watch STAGE=smoke
# or:  watch -n 2 cat /Users/raphaelmansuy/Github/03-working/edgequake/specs/001-benchmark/e2e/artifacts/smoke/LIVE.md
# or:  tail -f /Users/raphaelmansuy/Github/03-working/edgequake/specs/001-benchmark/e2e/artifacts/smoke/logs/progress.jsonl
```
