# bench001 LIVE — `medical-mid`

- **updated:** `2026-08-15T11:02:18Z`
- **started:** `2026-08-15T10:53:18Z`
- **run elapsed:** `9m00s`
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
| 2026-08-15T10:58:00Z | query_eq | running | EQ query 180/200 id=Medical-fc1f08f3 | 31s | 4m43s |
| 2026-08-15T10:58:03Z | query_parallel | running | EQ=running LR=running | — | 4m45s |
| 2026-08-15T10:58:04Z | query_lr | running | LR query 200/200 | 0s | 4m46s |
| 2026-08-15T10:58:17Z | query_eq | running | EQ query 190/200 id=Medical-e122bd14 | 16s | 5m00s |
| 2026-08-15T10:58:18Z | query_parallel | running | EQ=running LR=done | — | 5m00s |
| 2026-08-15T10:58:31Z | query_eq | running | EQ query 195/200 id=Medical-3aed2370 | 8s | 5m13s |
| 2026-08-15T10:58:33Z | query_parallel | running | EQ=running LR=done | — | 5m15s |
| 2026-08-15T10:58:38Z | query_eq | running | EQ query 200/200 id=Medical-a2ede728 | 0s | 5m21s |
| 2026-08-15T10:58:48Z | query_parallel | running | EQ=done LR=done | — | 5m30s |
| 2026-08-15T10:58:48Z | query_parallel | done | eq=200 lr=200 | — | 5m30s |
| 2026-08-15T11:02:18Z | score_parallel | running | EQ=done LR=done eval∥=24 | — | 9m00s |
| 2026-08-15T11:02:18Z | score_parallel | done | elapsed=3m30s | — | 9m00s |

## Monitor

```bash
make bench001-watch STAGE=medical-mid
# or:  watch -n 2 cat /Users/raphaelmansuy/Github/03-working/edgequake/specs/001-benchmark/e2e/artifacts/medical-mid/LIVE.md
# or:  tail -f /Users/raphaelmansuy/Github/03-working/edgequake/specs/001-benchmark/e2e/artifacts/medical-mid/logs/progress.jsonl
```
