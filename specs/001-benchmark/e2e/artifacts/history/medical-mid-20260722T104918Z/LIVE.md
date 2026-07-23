# bench001 LIVE — `medical-mid`

- **updated:** `2026-07-22T10:49:18Z`
- **started:** `2026-07-22T10:48:49Z`
- **run elapsed:** `18m16s`
- **phase:** `score_parallel` (done)
- **progress:** `50/50`
- **ETA (phase):** `0s`
- **detail:** elapsed=1m00s

## Corpus / chunking

- **docs:** `—`  (done `—`)
- **chunk size / overlap:** `—` / `—`
- **indexed chunks:** `—`
- **corpus chars:** `—`  capped=`None`
- **questions:** `—`

## Pipeline

○ prepare → ○ ingest_eq → ○ query_parallel → ✓ score_parallel → ○ report

## Recent ticks

| at (UTC) | phase | status | detail | eta | run |
|----------|-------|--------|--------|-----|-----|
| 2026-07-22T10:49:18Z | score_parallel | running | EQ=done LR=done eval∥=24 | — | 18m16s |
| 2026-07-22T10:49:18Z | score_parallel | done | elapsed=1m00s | — | 18m16s |

## Monitor

```bash
make bench001-watch STAGE=medical-mid
# or:  watch -n 2 cat /Users/raphaelmansuy/Github/03-working/edgequake/specs/001-benchmark/e2e/artifacts/medical-mid/LIVE.md
# or:  tail -f /Users/raphaelmansuy/Github/03-working/edgequake/specs/001-benchmark/e2e/artifacts/medical-mid/logs/progress.jsonl
```
