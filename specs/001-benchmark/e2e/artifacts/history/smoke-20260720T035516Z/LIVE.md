# bench001 LIVE — `smoke`

- **updated:** `2026-07-20T03:55:16Z`
- **started:** `2026-07-20T03:53:01Z`
- **run elapsed:** `2m15s`
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
| 2026-07-20T03:53:22Z | query_eq | running | EQ query 10/40 id=Medical-ba2c8007 | 1m04s | 21s |
| 2026-07-20T03:53:31Z | query_lr | running | LR query 40/40 | 0s | 30s |
| 2026-07-20T03:53:31Z | query_parallel | running | EQ=running LR=done | — | 30s |
| 2026-07-20T03:53:42Z | query_eq | running | EQ query 20/40 id=Medical-7b682af7 | 41s | 41s |
| 2026-07-20T03:53:46Z | query_parallel | running | EQ=running LR=done | — | 45s |
| 2026-07-20T03:54:00Z | query_eq | running | EQ query 30/40 id=Medical-8f9d5dde | 20s | 59s |
| 2026-07-20T03:54:16Z | query_parallel | running | EQ=running LR=done | — | 1m15s |
| 2026-07-20T03:54:27Z | query_eq | running | EQ query 40/40 id=Medical-fa8b9196 | 0s | 1m26s |
| 2026-07-20T03:54:31Z | query_parallel | running | EQ=done LR=done | — | 1m30s |
| 2026-07-20T03:54:31Z | query_parallel | done | eq=40 lr=40 | — | 1m30s |
| 2026-07-20T03:55:16Z | score_parallel | running | EQ=done LR=done eval∥=24 | — | 2m15s |
| 2026-07-20T03:55:16Z | score_parallel | done | elapsed=45s | — | 2m15s |

## Monitor

```bash
make bench001-watch STAGE=smoke
# or:  watch -n 2 cat /Users/raphaelmansuy/Github/03-working/edgequake/specs/001-benchmark/e2e/artifacts/smoke/LIVE.md
# or:  tail -f /Users/raphaelmansuy/Github/03-working/edgequake/specs/001-benchmark/e2e/artifacts/smoke/logs/progress.jsonl
```
