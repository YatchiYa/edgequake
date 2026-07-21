# bench001 LIVE — `smoke`

- **updated:** `2026-07-21T01:08:43Z`
- **started:** `2026-07-21T00:56:42Z`
- **run elapsed:** `12m00s`
- **phase:** `score_parallel` (done)
- **progress:** `10/10`
- **ETA (phase):** `0s`
- **detail:** elapsed=45s

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
| 2026-07-21T01:06:58Z | query_parallel | running | EQ=running LR=done | — | 10m15s |
| 2026-07-21T01:07:12Z | query_eq | running | EQ query 20/40 id=Medical-31580ac0 | 26s | 10m29s |
| 2026-07-21T01:07:13Z | query_parallel | running | EQ=running LR=done | — | 10m30s |
| 2026-07-21T01:07:26Z | query_eq | running | EQ query 30/40 id=Medical-8f9d5dde | 13s | 10m44s |
| 2026-07-21T01:07:28Z | query_parallel | running | EQ=running LR=done | — | 10m45s |
| 2026-07-21T01:07:36Z | query_eq | running | EQ query 35/40 id=Medical-d96c57fa | 7s | 10m53s |
| 2026-07-21T01:07:43Z | query_parallel | running | EQ=running LR=done | — | 11m00s |
| 2026-07-21T01:07:44Z | query_eq | running | EQ query 40/40 id=Medical-fa8b9196 | 0s | 11m02s |
| 2026-07-21T01:07:58Z | query_parallel | running | EQ=done LR=done | — | 11m15s |
| 2026-07-21T01:07:58Z | query_parallel | done | eq=40 lr=40 | — | 11m15s |
| 2026-07-21T01:08:43Z | score_parallel | running | EQ=done LR=done eval∥=8 | — | 12m00s |
| 2026-07-21T01:08:43Z | score_parallel | done | elapsed=45s | — | 12m00s |

## Monitor

```bash
make bench001-watch STAGE=smoke
# or:  watch -n 2 cat /Users/raphaelmansuy/Github/03-working/edgequake/specs/001-benchmark/e2e/artifacts/smoke/LIVE.md
# or:  tail -f /Users/raphaelmansuy/Github/03-working/edgequake/specs/001-benchmark/e2e/artifacts/smoke/logs/progress.jsonl
```
