# bench001 LIVE — `medical-mid`

- **updated:** `2026-08-15T12:03:23Z`
- **started:** `2026-08-15T11:54:53Z`
- **run elapsed:** `8m31s`
- **phase:** `score_parallel` (done)
- **progress:** `50/50`
- **ETA (phase):** `0s`
- **detail:** elapsed=3m30s

## Corpus / chunking

- **docs:** `1`  (done `—`)
- **chunk size / overlap:** `1200` / `100`
- **indexed chunks:** `—`
- **corpus chars:** `1052159`  capped=`False`
- **questions:** `200`

## Pipeline

✓ prepare → ○ ingest_eq → ✓ query_parallel → ✓ score_parallel → ○ report  |  ● query_lr ● query_eq

## Recent ticks

| at (UTC) | phase | status | detail | eta | run |
|----------|-------|--------|--------|-----|-----|
| 2026-08-15T11:59:23Z | query_parallel | running | EQ=running LR=running | — | 4m30s |
| 2026-08-15T11:59:28Z | query_lr | running | LR query 190/200 | 14s | 4m35s |
| 2026-08-15T11:59:28Z | query_eq | running | EQ query 190/200 id=Medical-e0a374bb | 14s | 4m36s |
| 2026-08-15T11:59:35Z | query_lr | running | LR query 195/200 | 7s | 4m43s |
| 2026-08-15T11:59:37Z | query_eq | running | EQ query 195/200 id=Medical-2a322545 | 7s | 4m44s |
| 2026-08-15T11:59:38Z | query_parallel | running | EQ=running LR=running | — | 4m45s |
| 2026-08-15T11:59:43Z | query_lr | running | LR query 200/200 | 0s | 4m50s |
| 2026-08-15T11:59:47Z | query_eq | running | EQ query 200/200 id=Medical-a2ede728 | 0s | 4m54s |
| 2026-08-15T11:59:53Z | query_parallel | running | EQ=done LR=done | — | 5m00s |
| 2026-08-15T11:59:53Z | query_parallel | done | eq=200 lr=200 | — | 5m00s |
| 2026-08-15T12:03:23Z | score_parallel | running | EQ=done LR=done eval∥=24 | — | 8m31s |
| 2026-08-15T12:03:23Z | score_parallel | done | elapsed=3m30s | — | 8m31s |

## Monitor

```bash
make bench001-watch STAGE=medical-mid
# or:  watch -n 2 cat /Users/raphaelmansuy/Github/03-working/edgequake/specs/001-benchmark/e2e/artifacts/medical-mid/LIVE.md
# or:  tail -f /Users/raphaelmansuy/Github/03-working/edgequake/specs/001-benchmark/e2e/artifacts/medical-mid/logs/progress.jsonl
```
