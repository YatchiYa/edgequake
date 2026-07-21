# bench001 LIVE — `smoke`

- **updated:** `2026-07-19T13:48:09Z`
- **started:** `2026-07-19T13:35:54Z`
- **run elapsed:** `12m15s`
- **phase:** `score_parallel` (done)
- **progress:** `10/10`
- **ETA (phase):** `0s`
- **detail:** elapsed=30s

## Corpus / chunking

- **docs:** `1`  (done `1`)
- **chunk size / overlap:** `1200` / `100`
- **indexed chunks:** `188`
- **corpus chars:** `1052159`  capped=`False`
- **questions:** `40`

## Pipeline

✓ prepare → ✓ ingest_eq → ✓ query_parallel → ✓ score_parallel → ○ report  |  ● query_lr ● query_eq

## Recent ticks

| at (UTC) | phase | status | detail | eta | run |
|----------|-------|--------|--------|-----|-----|
| 2026-07-19T13:46:50Z | ingest_eq | done | wall_s=655.7 | — | 10m56s |
| 2026-07-19T13:46:53Z | query_eq | running | EQ query 1/40 id=Medical-e7b5ec54 | 1m51s | 10m59s |
| 2026-07-19T13:46:54Z | query_parallel | running | EQ=running LR=done | — | 11m00s |
| 2026-07-19T13:47:06Z | query_eq | running | EQ query 15/40 id=Medical-641dcaf5 | 28s | 11m13s |
| 2026-07-19T13:47:09Z | query_parallel | running | EQ=running LR=done | — | 11m15s |
| 2026-07-19T13:47:19Z | query_eq | running | EQ query 25/40 id=Medical-6809b810 | 18s | 11m26s |
| 2026-07-19T13:47:24Z | query_parallel | running | EQ=running LR=done | — | 11m30s |
| 2026-07-19T13:47:39Z | query_eq | running | EQ query 40/40 id=Medical-fa8b9196 | 0s | 11m45s |
| 2026-07-19T13:47:39Z | query_parallel | running | EQ=done LR=done | — | 11m45s |
| 2026-07-19T13:47:39Z | query_parallel | done | eq=40 lr=40 | — | 11m45s |
| 2026-07-19T13:48:09Z | score_parallel | running | EQ=done LR=done eval∥=24 | — | 12m15s |
| 2026-07-19T13:48:09Z | score_parallel | done | elapsed=30s | — | 12m15s |

## Monitor

```bash
make bench001-watch STAGE=smoke
# or:  watch -n 2 cat /Users/raphaelmansuy/Github/03-working/edgequake/specs/001-benchmark/e2e/artifacts/smoke/LIVE.md
# or:  tail -f /Users/raphaelmansuy/Github/03-working/edgequake/specs/001-benchmark/e2e/artifacts/smoke/logs/progress.jsonl
```
