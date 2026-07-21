# bench001 LIVE — `smoke`

- **updated:** `2026-07-20T02:26:14Z`
- **started:** `2026-07-20T02:23:29Z`
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
| 2026-07-20T02:24:44Z | query_parallel | running | EQ=running LR=done | — | 1m15s |
| 2026-07-20T02:24:47Z | query_eq | running | EQ query 25/40 id=Medical-6809b810 | 47s | 1m18s |
| 2026-07-20T02:24:59Z | query_parallel | running | EQ=running LR=done | — | 1m30s |
| 2026-07-20T02:25:07Z | query_eq | running | EQ query 30/40 id=Medical-8f9d5dde | 33s | 1m38s |
| 2026-07-20T02:25:14Z | query_parallel | running | EQ=running LR=done | — | 1m45s |
| 2026-07-20T02:25:25Z | query_eq | running | EQ query 35/40 id=Medical-5242d398 | 17s | 1m56s |
| 2026-07-20T02:25:29Z | query_parallel | running | EQ=running LR=done | — | 2m00s |
| 2026-07-20T02:25:38Z | query_eq | running | EQ query 40/40 id=Medical-fa8b9196 | 0s | 2m10s |
| 2026-07-20T02:25:44Z | query_parallel | running | EQ=done LR=done | — | 2m15s |
| 2026-07-20T02:25:44Z | query_parallel | done | eq=40 lr=40 | — | 2m15s |
| 2026-07-20T02:26:14Z | score_parallel | running | EQ=done LR=done eval∥=24 | — | 2m45s |
| 2026-07-20T02:26:14Z | score_parallel | done | elapsed=30s | — | 2m45s |

## Monitor

```bash
make bench001-watch STAGE=smoke
# or:  watch -n 2 cat /Users/raphaelmansuy/Github/03-working/edgequake/specs/001-benchmark/e2e/artifacts/smoke/LIVE.md
# or:  tail -f /Users/raphaelmansuy/Github/03-working/edgequake/specs/001-benchmark/e2e/artifacts/smoke/logs/progress.jsonl
```
