# bench001 LIVE — `medical-mid`

- **updated:** `2026-08-07T08:18:48Z`
- **started:** `2026-08-07T08:15:51Z`
- **run elapsed:** `36m02s`
- **phase:** `report` (done)
- **progress:** `50/50`
- **ETA (phase):** `0s`
- **detail:** valid=True archive=medical-mid-20260807T081848Z

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
| 2026-08-07T08:18:47Z | score_parallel | running | EQ=done LR=done eval∥=24 | — | 36m01s |
| 2026-08-07T08:18:47Z | score_parallel | done | elapsed=4m16s | — | 36m01s |
| 2026-08-07T08:18:48Z | report | done | valid=True archive=medical-mid-20260807T081848Z | — | 36m02s |

## Monitor

```bash
make bench001-watch STAGE=medical-mid
# or:  watch -n 2 cat /Users/raphaelmansuy/Github/03-working/edgequake/specs/001-benchmark/e2e/artifacts/medical-mid/LIVE.md
# or:  tail -f /Users/raphaelmansuy/Github/03-working/edgequake/specs/001-benchmark/e2e/artifacts/medical-mid/logs/progress.jsonl
```
