# bench001 LIVE — `smoke`

- **updated:** `2026-07-20T01:44:52Z`
- **started:** `2026-07-20T01:42:06Z`
- **run elapsed:** `2m45s`
- **phase:** `score_parallel` (done)
- **progress:** `10/10`
- **ETA (phase):** `0s`
- **detail:** elapsed=1m00s

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
| 2026-07-20T01:42:51Z | query_parallel | running | EQ=running LR=done | — | 45s |
| 2026-07-20T01:43:02Z | query_eq | running | EQ query 25/40 id=Medical-6809b810 | 33s | 55s |
| 2026-07-20T01:43:06Z | query_parallel | running | EQ=running LR=done | — | 1m00s |
| 2026-07-20T01:43:12Z | query_eq | running | EQ query 30/40 id=Medical-8f9d5dde | 22s | 1m06s |
| 2026-07-20T01:43:21Z | query_parallel | running | EQ=running LR=done | — | 1m15s |
| 2026-07-20T01:43:32Z | query_eq | running | EQ query 35/40 id=Medical-d96c57fa | 12s | 1m26s |
| 2026-07-20T01:43:36Z | query_parallel | running | EQ=running LR=done | — | 1m30s |
| 2026-07-20T01:43:47Z | query_eq | running | EQ query 40/40 id=Medical-fa8b9196 | 0s | 1m41s |
| 2026-07-20T01:43:51Z | query_parallel | running | EQ=done LR=done | — | 1m45s |
| 2026-07-20T01:43:51Z | query_parallel | done | eq=40 lr=40 | — | 1m45s |
| 2026-07-20T01:44:52Z | score_parallel | running | EQ=done LR=done eval∥=24 | — | 2m45s |
| 2026-07-20T01:44:52Z | score_parallel | done | elapsed=1m00s | — | 2m45s |

## Monitor

```bash
make bench001-watch STAGE=smoke
# or:  watch -n 2 cat /Users/raphaelmansuy/Github/03-working/edgequake/specs/001-benchmark/e2e/artifacts/smoke/LIVE.md
# or:  tail -f /Users/raphaelmansuy/Github/03-working/edgequake/specs/001-benchmark/e2e/artifacts/smoke/logs/progress.jsonl
```
