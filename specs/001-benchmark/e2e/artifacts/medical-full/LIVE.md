# bench001 LIVE — `medical-full`

- **updated:** `2026-07-22T20:41:00Z`
- **started:** `2026-07-22T20:40:05Z`
- **run elapsed:** `1h10m51s`
- **phase:** `report` (done)
- **progress:** `1098/1098`
- **ETA (phase):** `0s`
- **detail:** valid=True archive=medical-full-20260722T204100Z

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
| 2026-07-22T20:40:59Z | score_parallel | running | EQ=done LR=done eval∥=24 | — | 1h10m51s |
| 2026-07-22T20:40:59Z | score_parallel | done | elapsed=5m15s | — | 1h10m51s |
| 2026-07-22T20:41:00Z | report | done | valid=True archive=medical-full-20260722T204100Z | — | 1h10m51s |

## Monitor

```bash
make bench001-watch STAGE=medical-full
# or:  watch -n 2 cat /Users/raphaelmansuy/Github/03-working/edgequake/specs/001-benchmark/e2e/artifacts/medical-full/LIVE.md
# or:  tail -f /Users/raphaelmansuy/Github/03-working/edgequake/specs/001-benchmark/e2e/artifacts/medical-full/logs/progress.jsonl
```
