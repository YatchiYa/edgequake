# bench001 LIVE — `smoke`

- **updated:** `2026-07-20T01:54:06Z`
- **started:** `2026-07-20T01:51:51Z`
- **run elapsed:** `2m15s`
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
| 2026-07-20T01:52:36Z | query_parallel | running | EQ=running LR=done | — | 45s |
| 2026-07-20T01:52:40Z | query_eq | running | EQ query 20/40 id=Medical-7b682af7 | 49s | 50s |
| 2026-07-20T01:52:51Z | query_parallel | running | EQ=running LR=done | — | 1m00s |
| 2026-07-20T01:52:55Z | query_eq | running | EQ query 25/40 id=Medical-6809b810 | 38s | 1m04s |
| 2026-07-20T01:53:06Z | query_parallel | running | EQ=running LR=done | — | 1m15s |
| 2026-07-20T01:53:08Z | query_eq | running | EQ query 30/40 id=Medical-8f9d5dde | 26s | 1m18s |
| 2026-07-20T01:53:21Z | query_parallel | running | EQ=running LR=done | — | 1m30s |
| 2026-07-20T01:53:34Z | query_eq | running | EQ query 40/40 id=Medical-fa8b9196 | 0s | 1m43s |
| 2026-07-20T01:53:36Z | query_parallel | running | EQ=done LR=done | — | 1m45s |
| 2026-07-20T01:53:36Z | query_parallel | done | eq=40 lr=40 | — | 1m45s |
| 2026-07-20T01:54:06Z | score_parallel | running | EQ=done LR=done eval∥=24 | — | 2m15s |
| 2026-07-20T01:54:06Z | score_parallel | done | elapsed=30s | — | 2m15s |

## Monitor

```bash
make bench001-watch STAGE=smoke
# or:  watch -n 2 cat /Users/raphaelmansuy/Github/03-working/edgequake/specs/001-benchmark/e2e/artifacts/smoke/LIVE.md
# or:  tail -f /Users/raphaelmansuy/Github/03-working/edgequake/specs/001-benchmark/e2e/artifacts/smoke/logs/progress.jsonl
```
