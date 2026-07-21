# bench001 LIVE — `smoke`

- **updated:** `2026-07-20T19:08:32Z`
- **started:** `2026-07-20T16:28:46Z`
- **run elapsed:** `11m16s`
- **phase:** `score_parallel` (done)
- **progress:** `10/10`
- **ETA (phase):** `0s`
- **detail:** elapsed=2m45s

## Corpus / chunking

- **docs:** `1`  (done `—`)
- **chunk size / overlap:** `1200` / `100`
- **indexed chunks:** `—`
- **corpus chars:** `1052159`  capped=`False`
- **questions:** `40`

## Pipeline

✓ prepare → ✗ ingest_eq → ✓ query_parallel → ✓ score_parallel → ○ report  |  ● query_lr

## Recent ticks

| at (UTC) | phase | status | detail | eta | run |
|----------|-------|--------|--------|-----|-----|
| 2026-07-20T18:27:35Z | query_parallel | running | EQ=running LR=done | — | 7m30s |
| 2026-07-20T18:27:40Z | ingest_eq | running | doc 1/1 ?/ pct=— eta=— | — | 7m36s |
| 2026-07-20T18:27:50Z | query_parallel | running | EQ=running LR=done | — | 7m45s |
| 2026-07-20T18:27:53Z | ingest_eq | running | doc 1/1 ?/ pct=— eta=— | — | 7m48s |
| 2026-07-20T18:28:05Z | query_parallel | running | EQ=running LR=done | — | 8m00s |
| 2026-07-20T18:31:14Z | ingest_eq | running | doc 1/1 ?/ pct=— eta=— | — | 8m13s |
| 2026-07-20T18:31:16Z | query_parallel | running | EQ=running LR=done | — | 8m15s |
| 2026-07-20T18:31:19Z | ingest_eq | failed | document 019f805b-c9ad-765c-aaa8-05c093d9aa6e not ready after 7200.0s (last_stat | — | 8m18s |
| 2026-07-20T18:31:31Z | query_parallel | running | EQ=done LR=done | — | 8m30s |
| 2026-07-20T18:31:31Z | query_parallel | done | eq=40 lr=40 | — | 8m30s |
| 2026-07-20T19:08:32Z | score_parallel | running | EQ=done LR=done eval∥=8 | — | 11m16s |
| 2026-07-20T19:08:32Z | score_parallel | done | elapsed=2m45s | — | 11m16s |

## Monitor

```bash
make bench001-watch STAGE=smoke
# or:  watch -n 2 cat /Users/raphaelmansuy/Github/03-working/edgequake/specs/001-benchmark/e2e/artifacts/smoke/LIVE.md
# or:  tail -f /Users/raphaelmansuy/Github/03-working/edgequake/specs/001-benchmark/e2e/artifacts/smoke/logs/progress.jsonl
```
