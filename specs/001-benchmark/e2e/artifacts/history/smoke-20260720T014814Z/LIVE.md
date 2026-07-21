# bench001 LIVE — `smoke`

- **updated:** `2026-07-20T01:48:14Z`
- **started:** `2026-07-20T01:45:29Z`
- **run elapsed:** `2m45s`
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
| 2026-07-20T01:46:29Z | query_parallel | running | EQ=running LR=done | — | 1m00s |
| 2026-07-20T01:46:41Z | query_eq | running | EQ query 25/40 id=Medical-6809b810 | 43s | 1m12s |
| 2026-07-20T01:46:59Z | query_parallel | running | EQ=running LR=done | — | 1m30s |
| 2026-07-20T01:47:03Z | query_eq | running | EQ query 30/40 id=Medical-8f9d5dde | 31s | 1m34s |
| 2026-07-20T01:47:14Z | query_parallel | running | EQ=running LR=done | — | 1m45s |
| 2026-07-20T01:47:25Z | query_eq | running | EQ query 35/40 id=Medical-d96c57fa | 17s | 1m56s |
| 2026-07-20T01:47:29Z | query_parallel | running | EQ=running LR=done | — | 2m00s |
| 2026-07-20T01:47:40Z | query_eq | running | EQ query 40/40 id=Medical-c8a65fec | 0s | 2m11s |
| 2026-07-20T01:47:44Z | query_parallel | running | EQ=done LR=done | — | 2m15s |
| 2026-07-20T01:47:44Z | query_parallel | done | eq=40 lr=40 | — | 2m15s |
| 2026-07-20T01:48:14Z | score_parallel | running | EQ=done LR=done eval∥=24 | — | 2m45s |
| 2026-07-20T01:48:14Z | score_parallel | done | elapsed=30s | — | 2m45s |

## Monitor

```bash
make bench001-watch STAGE=smoke
# or:  watch -n 2 cat /Users/raphaelmansuy/Github/03-working/edgequake/specs/001-benchmark/e2e/artifacts/smoke/LIVE.md
# or:  tail -f /Users/raphaelmansuy/Github/03-working/edgequake/specs/001-benchmark/e2e/artifacts/smoke/logs/progress.jsonl
```
