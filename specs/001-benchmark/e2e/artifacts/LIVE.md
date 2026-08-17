<<<<<<< HEAD
# bench001 LIVE — `smoke`

- **updated:** `2026-07-23T13:44:53Z`
- **started:** `2026-07-23T13:42:22Z`
- **run elapsed:** `2m30s`
- **phase:** `report` (done)
- **progress:** `10/10`
- **ETA (phase):** `0s`
- **detail:** valid=True archive=smoke-20260723T134452Z

## Corpus / chunking

- **docs:** `1`  (done `—`)
- **chunk size / overlap:** `1200` / `100`
- **indexed chunks:** `—`
- **corpus chars:** `1052159`  capped=`False`
- **questions:** `40`

## Pipeline

✓ prepare → ○ ingest_eq → ✓ query_parallel → ✓ score_parallel → ✓ report  |  ● query_eq ● query_lr
=======
# bench001 LIVE — `medical-mid`

- **updated:** `2026-08-02T15:39:28Z`
- **started:** `2026-08-02T15:38:45Z`
- **run elapsed:** `4m46s`
- **phase:** `report` (done)
- **progress:** `50/50`
- **ETA (phase):** `0s`
- **detail:** valid=True archive=medical-mid-20260802T153928Z

## Corpus / chunking

- **docs:** `—`  (done `—`)
- **chunk size / overlap:** `—` / `—`
- **indexed chunks:** `—`
- **corpus chars:** `—`  capped=`None`
- **questions:** `—`

## Pipeline

○ prepare → ○ ingest_eq → ○ query_parallel → ✓ score_parallel → ✓ report
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042

## Recent ticks

| at (UTC) | phase | status | detail | eta | run |
|----------|-------|--------|--------|-----|-----|
<<<<<<< HEAD
| 2026-07-23T13:43:20Z | query_eq | running | EQ query 40/40 id=Medical-fa8b9196 | 0s | 58s |
| 2026-07-23T13:43:22Z | query_parallel | running | EQ=done LR=running | — | 1m00s |
| 2026-07-23T13:43:37Z | query_lr | running | LR query 30/40 | 25s | 1m15s |
| 2026-07-23T13:43:52Z | query_parallel | running | EQ=done LR=running | — | 1m30s |
| 2026-07-23T13:44:00Z | query_lr | running | LR query 35/40 | 14s | 1m37s |
| 2026-07-23T13:44:07Z | query_parallel | running | EQ=done LR=running | — | 1m45s |
| 2026-07-23T13:44:12Z | query_lr | running | LR query 40/40 | 0s | 1m50s |
| 2026-07-23T13:44:22Z | query_parallel | running | EQ=done LR=done | — | 2m00s |
| 2026-07-23T13:44:22Z | query_parallel | done | eq=40 lr=40 | — | 2m00s |
| 2026-07-23T13:44:52Z | score_parallel | running | EQ=done LR=done eval∥=24 | — | 2m30s |
| 2026-07-23T13:44:52Z | score_parallel | done | elapsed=30s | — | 2m30s |
| 2026-07-23T13:44:53Z | report | done | valid=True archive=smoke-20260723T134452Z | — | 2m30s |
=======
| 2026-08-02T15:39:28Z | score_parallel | running | EQ=done LR=done eval∥=24 | — | 4m45s |
| 2026-08-02T15:39:28Z | score_parallel | done | elapsed=1m00s | — | 4m45s |
| 2026-08-02T15:39:28Z | report | done | valid=True archive=medical-mid-20260802T153928Z | — | 4m46s |
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042

## Monitor

```bash
<<<<<<< HEAD
make bench001-watch STAGE=smoke
# or:  watch -n 2 cat /Users/raphaelmansuy/Github/03-working/edgequake/specs/001-benchmark/e2e/artifacts/smoke/LIVE.md
# or:  tail -f /Users/raphaelmansuy/Github/03-working/edgequake/specs/001-benchmark/e2e/artifacts/smoke/logs/progress.jsonl
=======
make bench001-watch STAGE=medical-mid
# or:  watch -n 2 cat /Users/raphaelmansuy/Github/03-working/edgequake/specs/001-benchmark/e2e/artifacts/medical-mid/LIVE.md
# or:  tail -f /Users/raphaelmansuy/Github/03-working/edgequake/specs/001-benchmark/e2e/artifacts/medical-mid/logs/progress.jsonl
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
```
