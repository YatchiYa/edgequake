# bench001 LIVE — `smoke`

- **updated:** `2026-07-19T13:50:43Z`
- **started:** `2026-07-19T13:48:58Z`
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

✓ prepare → ○ ingest_eq → ✓ query_parallel → ✓ score_parallel → ○ report  |  ● query_lr

## Recent ticks

| at (UTC) | phase | status | detail | eta | run |
|----------|-------|--------|--------|-----|-----|
| 2026-07-19T13:48:58Z | prepare | done | n_questions=40 docs=1 chars=1052159 chunk=1200/100 | — | 0s |
| 2026-07-19T13:48:58Z | query_parallel | running | EQ=running LR=running | — | 0s |
| 2026-07-19T13:49:08Z | query_lr | running | LR query 10/40 | 27s | 10s |
| 2026-07-19T13:49:13Z | query_parallel | running | EQ=running LR=running | — | 15s |
| 2026-07-19T13:49:28Z | query_lr | running | LR query 30/40 | 10s | 30s |
| 2026-07-19T13:49:28Z | query_parallel | running | EQ=running LR=running | — | 30s |
| 2026-07-19T13:49:38Z | query_lr | running | LR query 40/40 | 0s | 40s |
| 2026-07-19T13:50:13Z | query_parallel | running | EQ=done LR=done | — | 1m15s |
| 2026-07-19T13:50:13Z | query_parallel | done | eq=40 lr=40 | — | 1m15s |
| 2026-07-19T13:50:43Z | score_parallel | running | EQ=done LR=done eval∥=24 | — | 1m45s |
| 2026-07-19T13:50:43Z | score_parallel | done | elapsed=30s | — | 1m45s |

## Monitor

```bash
make bench001-watch STAGE=smoke
# or:  watch -n 2 cat /Users/raphaelmansuy/Github/03-working/edgequake/specs/001-benchmark/e2e/artifacts/smoke/LIVE.md
# or:  tail -f /Users/raphaelmansuy/Github/03-working/edgequake/specs/001-benchmark/e2e/artifacts/smoke/logs/progress.jsonl
```
