# bench001 LIVE — `smoke`

- **updated:** `2026-07-20T06:24:28Z`
- **started:** `2026-07-20T06:21:43Z`
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
| 2026-07-20T06:22:28Z | query_parallel | running | EQ=running LR=done | — | 45s |
| 2026-07-20T06:22:28Z | query_eq | running | EQ query 15/40 id=Medical-2ca88e8f | 1m16s | 46s |
| 2026-07-20T06:22:43Z | query_parallel | running | EQ=running LR=done | — | 1m00s |
| 2026-07-20T06:22:54Z | query_eq | running | EQ query 25/40 id=Medical-6809b810 | 43s | 1m11s |
| 2026-07-20T06:22:58Z | query_parallel | running | EQ=running LR=done | — | 1m15s |
| 2026-07-20T06:23:11Z | query_eq | running | EQ query 30/40 id=Medical-1991db28 | 29s | 1m28s |
| 2026-07-20T06:23:28Z | query_parallel | running | EQ=running LR=done | — | 1m45s |
| 2026-07-20T06:23:40Z | query_eq | running | EQ query 40/40 id=Medical-fa8b9196 | 0s | 1m58s |
| 2026-07-20T06:23:43Z | query_parallel | running | EQ=done LR=done | — | 2m00s |
| 2026-07-20T06:23:43Z | query_parallel | done | eq=40 lr=40 | — | 2m00s |
| 2026-07-20T06:24:28Z | score_parallel | running | EQ=done LR=done eval∥=16 | — | 2m45s |
| 2026-07-20T06:24:28Z | score_parallel | done | elapsed=45s | — | 2m45s |

## Monitor

```bash
make bench001-watch STAGE=smoke
# or:  watch -n 2 cat /Users/raphaelmansuy/Github/03-working/edgequake/specs/001-benchmark/e2e/artifacts/smoke/LIVE.md
# or:  tail -f /Users/raphaelmansuy/Github/03-working/edgequake/specs/001-benchmark/e2e/artifacts/smoke/logs/progress.jsonl
```
