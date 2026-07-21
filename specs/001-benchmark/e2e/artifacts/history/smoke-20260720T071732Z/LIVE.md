# bench001 LIVE — `smoke`

- **updated:** `2026-07-20T07:17:32Z`
- **started:** `2026-07-20T07:13:32Z`
- **run elapsed:** `4m01s`
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

✓ prepare → ○ ingest_eq → ✓ query_parallel → ✓ score_parallel → ○ report  |  ● query_eq ● query_lr

## Recent ticks

| at (UTC) | phase | status | detail | eta | run |
|----------|-------|--------|--------|-----|-----|
| 2026-07-20T07:15:02Z | query_parallel | running | EQ=running LR=done | — | 1m30s |
| 2026-07-20T07:15:14Z | query_eq | running | EQ query 25/40 id=Medical-6809b810 | 1m01s | 1m42s |
| 2026-07-20T07:15:17Z | query_parallel | running | EQ=running LR=done | — | 1m46s |
| 2026-07-20T07:15:30Z | query_eq | running | EQ query 30/40 id=Medical-25f9adbb | 39s | 1m59s |
| 2026-07-20T07:16:02Z | query_parallel | running | EQ=running LR=done | — | 2m31s |
| 2026-07-20T07:16:06Z | query_eq | running | EQ query 35/40 id=Medical-5242d398 | 22s | 2m35s |
| 2026-07-20T07:16:17Z | query_parallel | running | EQ=running LR=done | — | 2m46s |
| 2026-07-20T07:16:31Z | query_eq | running | EQ query 40/40 id=Medical-c8a65fec | 0s | 2m59s |
| 2026-07-20T07:16:32Z | query_parallel | running | EQ=done LR=done | — | 3m01s |
| 2026-07-20T07:16:32Z | query_parallel | done | eq=40 lr=40 | — | 3m01s |
| 2026-07-20T07:17:32Z | score_parallel | running | EQ=done LR=done eval∥=8 | — | 4m01s |
| 2026-07-20T07:17:32Z | score_parallel | done | elapsed=1m00s | — | 4m01s |

## Monitor

```bash
make bench001-watch STAGE=smoke
# or:  watch -n 2 cat /Users/raphaelmansuy/Github/03-working/edgequake/specs/001-benchmark/e2e/artifacts/smoke/LIVE.md
# or:  tail -f /Users/raphaelmansuy/Github/03-working/edgequake/specs/001-benchmark/e2e/artifacts/smoke/logs/progress.jsonl
```
