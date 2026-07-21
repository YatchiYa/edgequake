# bench001 LIVE — `smoke`

- **updated:** `2026-07-20T11:19:44Z`
- **started:** `2026-07-20T11:17:58Z`
- **run elapsed:** `1m45s`
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
| 2026-07-20T11:18:26Z | query_eq | running | EQ query 15/40 id=Medical-641dcaf5 | 46s | 27s |
| 2026-07-20T11:18:28Z | query_lr | running | LR query 40/40 | 0s | 30s |
| 2026-07-20T11:18:28Z | query_parallel | running | EQ=running LR=done | — | 30s |
| 2026-07-20T11:18:41Z | query_eq | running | EQ query 25/40 id=Medical-6809b810 | 25s | 42s |
| 2026-07-20T11:18:43Z | query_parallel | running | EQ=running LR=done | — | 45s |
| 2026-07-20T11:18:50Z | query_eq | running | EQ query 30/40 id=Medical-8f9d5dde | 17s | 52s |
| 2026-07-20T11:18:58Z | query_parallel | running | EQ=running LR=done | — | 1m00s |
| 2026-07-20T11:19:09Z | query_eq | running | EQ query 40/40 id=Medical-fa8b9196 | 0s | 1m10s |
| 2026-07-20T11:19:13Z | query_parallel | running | EQ=done LR=done | — | 1m15s |
| 2026-07-20T11:19:13Z | query_parallel | done | eq=40 lr=40 | — | 1m15s |
| 2026-07-20T11:19:44Z | score_parallel | running | EQ=done LR=done eval∥=8 | — | 1m45s |
| 2026-07-20T11:19:44Z | score_parallel | done | elapsed=30s | — | 1m45s |

## Monitor

```bash
make bench001-watch STAGE=smoke
# or:  watch -n 2 cat /Users/raphaelmansuy/Github/03-working/edgequake/specs/001-benchmark/e2e/artifacts/smoke/LIVE.md
# or:  tail -f /Users/raphaelmansuy/Github/03-working/edgequake/specs/001-benchmark/e2e/artifacts/smoke/logs/progress.jsonl
```
