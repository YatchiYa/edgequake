# bench001 LIVE — `smoke`

- **updated:** `2026-07-19T14:28:41Z`
- **started:** `2026-07-19T14:26:11Z`
- **run elapsed:** `2m30s`
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
| 2026-07-19T14:27:01Z | query_eq | running | EQ query 20/40 id=Medical-31580ac0 | 50s | 50s |
| 2026-07-19T14:27:07Z | query_lr | running | LR query 35/40 | 8s | 56s |
| 2026-07-19T14:27:11Z | query_parallel | running | EQ=running LR=running | — | 1m00s |
| 2026-07-19T14:27:13Z | query_eq | running | EQ query 25/40 id=Medical-6809b810 | 37s | 1m02s |
| 2026-07-19T14:27:15Z | query_lr | running | LR query 40/40 | 0s | 1m04s |
| 2026-07-19T14:27:24Z | query_eq | running | EQ query 30/40 id=Medical-8f9d5dde | 24s | 1m13s |
| 2026-07-19T14:27:41Z | query_parallel | running | EQ=running LR=done | — | 1m30s |
| 2026-07-19T14:27:54Z | query_eq | running | EQ query 40/40 id=Medical-c8a65fec | 0s | 1m43s |
| 2026-07-19T14:27:56Z | query_parallel | running | EQ=done LR=done | — | 1m45s |
| 2026-07-19T14:27:56Z | query_parallel | done | eq=40 lr=40 | — | 1m45s |
| 2026-07-19T14:28:41Z | score_parallel | running | EQ=done LR=done eval∥=24 | — | 2m30s |
| 2026-07-19T14:28:41Z | score_parallel | done | elapsed=45s | — | 2m30s |

## Monitor

```bash
make bench001-watch STAGE=smoke
# or:  watch -n 2 cat /Users/raphaelmansuy/Github/03-working/edgequake/specs/001-benchmark/e2e/artifacts/smoke/LIVE.md
# or:  tail -f /Users/raphaelmansuy/Github/03-working/edgequake/specs/001-benchmark/e2e/artifacts/smoke/logs/progress.jsonl
```
