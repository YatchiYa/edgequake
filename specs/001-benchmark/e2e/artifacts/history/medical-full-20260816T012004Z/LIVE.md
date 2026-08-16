# bench001 LIVE — `medical-full`

- **updated:** `2026-08-16T01:20:04Z`
- **started:** `2026-08-16T01:19:36Z`
- **run elapsed:** `44m49s`
- **phase:** `score_parallel` (done)
- **progress:** `1098/1098`
- **ETA (phase):** `0s`
- **detail:** elapsed=9m46s

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
| 2026-08-16T01:20:04Z | score_parallel | running | EQ=done LR=done eval∥=24 | — | 44m49s |
| 2026-08-16T01:20:04Z | score_parallel | done | elapsed=9m46s | — | 44m49s |

## Monitor

```bash
make bench001-watch STAGE=medical-full
# or:  watch -n 2 cat /Users/raphaelmansuy/Github/03-working/edgequake/specs/001-benchmark/e2e/artifacts/medical-full/LIVE.md
# or:  tail -f /Users/raphaelmansuy/Github/03-working/edgequake/specs/001-benchmark/e2e/artifacts/medical-full/logs/progress.jsonl
```
