# bench001 LIVE — `smoke`

- **updated:** `2026-07-20T01:59:51Z`
- **started:** `2026-07-20T01:57:20Z`
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
| 2026-07-20T01:58:06Z | query_parallel | running | EQ=running LR=done | — | 45s |
| 2026-07-20T01:58:09Z | query_eq | running | EQ query 25/40 id=Medical-6809b810 | 29s | 48s |
| 2026-07-20T01:58:21Z | query_parallel | running | EQ=running LR=done | — | 1m00s |
| 2026-07-20T01:58:24Z | query_eq | running | EQ query 30/40 id=Medical-8f9d5dde | 21s | 1m03s |
| 2026-07-20T01:58:36Z | query_parallel | running | EQ=running LR=done | — | 1m15s |
| 2026-07-20T01:58:37Z | query_eq | running | EQ query 35/40 id=Medical-0c5272d1 | 11s | 1m17s |
| 2026-07-20T01:58:51Z | query_parallel | running | EQ=running LR=done | — | 1m30s |
| 2026-07-20T01:58:51Z | query_eq | running | EQ query 40/40 id=Medical-c8a65fec | 0s | 1m30s |
| 2026-07-20T01:59:06Z | query_parallel | running | EQ=done LR=done | — | 1m45s |
| 2026-07-20T01:59:06Z | query_parallel | done | eq=40 lr=40 | — | 1m45s |
| 2026-07-20T01:59:51Z | score_parallel | running | EQ=done LR=done eval∥=24 | — | 2m30s |
| 2026-07-20T01:59:51Z | score_parallel | done | elapsed=45s | — | 2m30s |

## Monitor

```bash
make bench001-watch STAGE=smoke
# or:  watch -n 2 cat /Users/raphaelmansuy/Github/03-working/edgequake/specs/001-benchmark/e2e/artifacts/smoke/LIVE.md
# or:  tail -f /Users/raphaelmansuy/Github/03-working/edgequake/specs/001-benchmark/e2e/artifacts/smoke/logs/progress.jsonl
```
