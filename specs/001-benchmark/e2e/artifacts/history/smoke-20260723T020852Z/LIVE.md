# bench001 LIVE — `smoke`

- **updated:** `2026-07-23T02:08:52Z`
- **started:** `2026-07-23T01:57:52Z`
- **run elapsed:** `11m00s`
- **phase:** `score_parallel` (done)
- **progress:** `10/10`
- **ETA (phase):** `0s`
- **detail:** elapsed=30s

## Corpus / chunking

- **docs:** `1`  (done `1`)
- **chunk size / overlap:** `1200` / `100`
- **indexed chunks:** `—`
- **corpus chars:** `1052159`  capped=`False`
- **questions:** `40`

## Pipeline

✓ prepare → ✓ ingest_eq → ✓ query_parallel → ✓ score_parallel → ○ report  |  ● query_lr ● query_eq

## Recent ticks

| at (UTC) | phase | status | detail | eta | run |
|----------|-------|--------|--------|-----|-----|
| 2026-07-23T02:07:29Z | ingest_eq | done | wall_s=576.6 | — | 9m37s |
| 2026-07-23T02:07:34Z | query_eq | running | EQ query 5/40 id=Medical-c6d69844 | 40s | 9m42s |
| 2026-07-23T02:07:37Z | query_parallel | running | EQ=running LR=done | — | 9m45s |
| 2026-07-23T02:07:52Z | query_eq | running | EQ query 25/40 id=Medical-6809b810 | 14s | 10m00s |
| 2026-07-23T02:07:52Z | query_parallel | running | EQ=running LR=done | — | 10m00s |
| 2026-07-23T02:08:05Z | query_eq | running | EQ query 35/40 id=Medical-0c5272d1 | 5s | 10m13s |
| 2026-07-23T02:08:07Z | query_parallel | running | EQ=running LR=done | — | 10m15s |
| 2026-07-23T02:08:11Z | query_eq | running | EQ query 40/40 id=Medical-fa8b9196 | 0s | 10m19s |
| 2026-07-23T02:08:22Z | query_parallel | running | EQ=done LR=done | — | 10m30s |
| 2026-07-23T02:08:22Z | query_parallel | done | eq=40 lr=40 | — | 10m30s |
| 2026-07-23T02:08:52Z | score_parallel | running | EQ=done LR=done eval∥=24 | — | 11m00s |
| 2026-07-23T02:08:52Z | score_parallel | done | elapsed=30s | — | 11m00s |

## Monitor

```bash
make bench001-watch STAGE=smoke
# or:  watch -n 2 cat /Users/raphaelmansuy/Github/03-working/edgequake/specs/001-benchmark/e2e/artifacts/smoke/LIVE.md
# or:  tail -f /Users/raphaelmansuy/Github/03-working/edgequake/specs/001-benchmark/e2e/artifacts/smoke/logs/progress.jsonl
```
