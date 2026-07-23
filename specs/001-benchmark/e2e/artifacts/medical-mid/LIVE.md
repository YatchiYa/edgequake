# bench001 LIVE — `medical-mid`

- **updated:** `2026-07-23T13:41:24Z`
- **started:** `2026-07-23T13:40:46Z`
- **run elapsed:** `15m31s`
- **phase:** `report` (done)
- **progress:** `50/50`
- **ETA (phase):** `0s`
- **detail:** valid=True archive=medical-mid-20260723T134124Z

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
| 2026-07-23T13:41:24Z | score_parallel | running | EQ=done LR=done eval∥=24 | — | 15m31s |
| 2026-07-23T13:41:24Z | score_parallel | done | elapsed=1m00s | — | 15m31s |
| 2026-07-23T13:41:24Z | report | done | valid=True archive=medical-mid-20260723T134124Z | — | 15m31s |

## Monitor

```bash
make bench001-watch STAGE=medical-mid
# or:  watch -n 2 cat /Users/raphaelmansuy/Github/03-working/edgequake/specs/001-benchmark/e2e/artifacts/medical-mid/LIVE.md
# or:  tail -f /Users/raphaelmansuy/Github/03-working/edgequake/specs/001-benchmark/e2e/artifacts/medical-mid/logs/progress.jsonl
```
