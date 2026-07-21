# bench001 LIVE — `smoke`

- **updated:** `2026-07-20T03:28:29Z`
- **started:** `2026-07-20T03:25:44Z`
- **run elapsed:** `2m45s`
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
| 2026-07-20T03:26:44Z | query_parallel | running | EQ=running LR=done | — | 1m00s |
| 2026-07-20T03:26:49Z | query_eq | running | EQ query 25/40 id=Medical-6809b810 | 39s | 1m05s |
| 2026-07-20T03:26:59Z | query_parallel | running | EQ=running LR=done | — | 1m15s |
| 2026-07-20T03:27:09Z | query_eq | running | EQ query 30/40 id=Medical-8f9d5dde | 28s | 1m25s |
| 2026-07-20T03:27:14Z | query_parallel | running | EQ=running LR=done | — | 1m30s |
| 2026-07-20T03:27:22Z | query_eq | running | EQ query 35/40 id=Medical-0c5272d1 | 14s | 1m38s |
| 2026-07-20T03:27:29Z | query_parallel | running | EQ=running LR=done | — | 1m45s |
| 2026-07-20T03:27:39Z | query_eq | running | EQ query 40/40 id=Medical-c8a65fec | 0s | 1m55s |
| 2026-07-20T03:27:44Z | query_parallel | running | EQ=done LR=done | — | 2m00s |
| 2026-07-20T03:27:44Z | query_parallel | done | eq=40 lr=40 | — | 2m00s |
| 2026-07-20T03:28:29Z | score_parallel | running | EQ=done LR=done eval∥=24 | — | 2m45s |
| 2026-07-20T03:28:29Z | score_parallel | done | elapsed=45s | — | 2m45s |

## Monitor

```bash
make bench001-watch STAGE=smoke
# or:  watch -n 2 cat /Users/raphaelmansuy/Github/03-working/edgequake/specs/001-benchmark/e2e/artifacts/smoke/LIVE.md
# or:  tail -f /Users/raphaelmansuy/Github/03-working/edgequake/specs/001-benchmark/e2e/artifacts/smoke/logs/progress.jsonl
```
