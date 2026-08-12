# bench001 LIVE — `medical-mid`

- **updated:** `2026-08-12T00:42:16Z`
- **started:** `2026-08-12T00:41:47Z`
- **run elapsed:** `15m01s`
- **phase:** `report` (done)
- **progress:** `50/50`
- **ETA (phase):** `0s`
- **detail:** valid=True archive=medical-mid-20260812T004216Z

## Corpus / chunking

- **docs:** `—`  (done `—`)
- **chunk size / overlap:** `—` / `—`
- **indexed chunks:** `—`
- **corpus chars:** `—`  capped=`None`
- **questions:** `—`

## Pipeline

○ prepare → ○ ingest_eq → ○ query_parallel → ✓ score_parallel → ✓ report

## Recent ticks

| at (UTC) | phase | status | detail | eta | run |
|----------|-------|--------|--------|-----|-----|
| 2026-08-12T00:42:16Z | score_parallel | running | EQ=done LR=done eval∥=24 | — | 15m01s |
| 2026-08-12T00:42:16Z | score_parallel | done | elapsed=45s | — | 15m01s |
| 2026-08-12T00:42:16Z | report | done | valid=True archive=medical-mid-20260812T004216Z | — | 15m01s |

## Monitor

```bash
make bench001-watch STAGE=medical-mid
# or:  watch -n 2 cat /Users/raphaelmansuy/Github/03-working/edgequake/specs/001-benchmark/e2e/artifacts/medical-mid/LIVE.md
# or:  tail -f /Users/raphaelmansuy/Github/03-working/edgequake/specs/001-benchmark/e2e/artifacts/medical-mid/logs/progress.jsonl
```
