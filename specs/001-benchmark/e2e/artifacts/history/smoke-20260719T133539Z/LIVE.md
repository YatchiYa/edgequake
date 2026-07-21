# bench001 LIVE — `smoke`

- **updated:** `2026-07-19T13:35:39Z`
- **started:** `2026-07-19T13:34:24Z`
- **run elapsed:** `1m15s`
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

✓ prepare → ○ ingest_eq → ✓ query_parallel → ✓ score_parallel → ○ report  |  ● query_lr

## Recent ticks

| at (UTC) | phase | status | detail | eta | run |
|----------|-------|--------|--------|-----|-----|
| 2026-07-19T13:34:24Z | prepare | done | n_questions=40 docs=1 chars=1052159 chunk=1200/100 | — | 0s |
| 2026-07-19T13:34:24Z | query_parallel | running | EQ=running LR=running | — | 0s |
| 2026-07-19T13:34:37Z | query_lr | running | LR query 15/40 | 20s | 13s |
| 2026-07-19T13:34:39Z | query_parallel | running | EQ=done LR=running | — | 15s |
| 2026-07-19T13:34:51Z | query_lr | running | LR query 35/40 | 4s | 27s |
| 2026-07-19T13:34:54Z | query_parallel | running | EQ=done LR=running | — | 30s |
| 2026-07-19T13:34:55Z | query_lr | running | LR query 40/40 | 0s | 31s |
| 2026-07-19T13:35:09Z | query_parallel | running | EQ=done LR=done | — | 45s |
| 2026-07-19T13:35:09Z | query_parallel | done | eq=40 lr=40 | — | 45s |
| 2026-07-19T13:35:39Z | score_parallel | running | EQ=done LR=done eval∥=24 | — | 1m15s |
| 2026-07-19T13:35:39Z | score_parallel | done | elapsed=30s | — | 1m15s |

## Monitor

```bash
make bench001-watch STAGE=smoke
# or:  watch -n 2 cat /Users/raphaelmansuy/Github/03-working/edgequake/specs/001-benchmark/e2e/artifacts/smoke/LIVE.md
# or:  tail -f /Users/raphaelmansuy/Github/03-working/edgequake/specs/001-benchmark/e2e/artifacts/smoke/logs/progress.jsonl
```
