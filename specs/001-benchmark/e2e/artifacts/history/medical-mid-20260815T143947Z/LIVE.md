# bench001 LIVE — `medical-mid`

- **updated:** `2026-08-15T14:39:46Z`
- **started:** `2026-08-15T14:34:46Z`
- **run elapsed:** `5m01s`
- **phase:** `score_parallel` (done)
- **progress:** `50/50`
- **ETA (phase):** `0s`
- **detail:** elapsed=45s

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
| 2026-08-15T14:38:01Z | query_parallel | running | EQ=running LR=done | — | 3m15s |
| 2026-08-15T14:38:08Z | query_eq | running | EQ query 175/200 id=Medical-d8e24b59 | 29s | 3m22s |
| 2026-08-15T14:38:16Z | query_parallel | running | EQ=running LR=done | — | 3m30s |
| 2026-08-15T14:38:24Z | query_eq | running | EQ query 185/200 id=Medical-83c906e0 | 18s | 3m38s |
| 2026-08-15T14:38:31Z | query_parallel | running | EQ=running LR=done | — | 3m45s |
| 2026-08-15T14:38:39Z | query_eq | running | EQ query 195/200 id=Medical-3aed2370 | 6s | 3m53s |
| 2026-08-15T14:38:46Z | query_parallel | running | EQ=running LR=done | — | 4m00s |
| 2026-08-15T14:38:47Z | query_eq | running | EQ query 200/200 id=Medical-65be7ca5 | 0s | 4m01s |
| 2026-08-15T14:39:01Z | query_parallel | running | EQ=done LR=done | — | 4m15s |
| 2026-08-15T14:39:01Z | query_parallel | done | eq=200 lr=200 | — | 4m15s |
| 2026-08-15T14:39:46Z | score_parallel | running | EQ=done LR=done eval∥=24 | — | 5m01s |
| 2026-08-15T14:39:46Z | score_parallel | done | elapsed=45s | — | 5m01s |

## Monitor

```bash
make bench001-watch STAGE=medical-mid
# or:  watch -n 2 cat /Users/raphaelmansuy/Github/03-working/edgequake/specs/001-benchmark/e2e/artifacts/medical-mid/LIVE.md
# or:  tail -f /Users/raphaelmansuy/Github/03-working/edgequake/specs/001-benchmark/e2e/artifacts/medical-mid/logs/progress.jsonl
```
