# SPEC-063 — Proof ladder

Do not raise FAQ/docs claims past the highest green rung.

## Vector / ANN ladder

| Step       | Corpus         | Pass criteria                                                                                                                               | Posture                                    |
| ------------| ----------------| ---------------------------------------------------------------------------------------------------------------------------------------------| --------------------------------------------|
| **Proven** | 50k @1536      | SPEC-061/062 prod stress + Q1-d                                                                                                             | `make data-access-perf-matrix-prod`        |
| **L1**     | **100k** @1536 | Q1-d via SPEC-064 battle (`halfvec` + partial WS); cold exact-scan cliff still documented | `make ann-scale-battle` / capacity ladder |
| **L2**     | **500k**       | Same + document `ensure_ann_index` wall + RAM note; cliff 10s                                                                               | `EDGEQUAKE_CAPACITY_LADDER=L2` manual soak |
| **L3**     | **1M**         | Meet Q1-d **or** archive FORBIDDEN cliff (p95 + host RAM); cliff 20s                                                                        | `EDGEQUAKE_CAPACITY_LADDER=L3` optional    |

**Promotion:** only promote “supported at Q1-d” when `capacity_ladder_filtered_ann_single.pass=true` (p95&lt;500ms). A completed ladder with `slo_pass=false` is a **measured cliff**, not a support claim.

Env: `EDGEQUAKE_PERF_SCALE=large` selects Large stress tables; ladder step via `EDGEQUAKE_CAPACITY_LADDER=L1|L2|L3` (default L1).

## Graph ladder (separate)

| Step | Corpus | Pass criteria |
|------|--------|---------------|
| G0 | 1k nodes / 1k edges | SPEC-061 edge upsert + degrees |
| G1 | 100k nodes (store + degrees sample) | upsert batches complete; degrees p95 &lt;100ms sample |
| G2 | 500k nodes | same + migration/index notes |
| Community | ≤50k | API hard gate remains unless ops override + soak |

## Artifact contract

- JSONL / log under [`e2e/artifacts/`](e2e/artifacts/)
- `RUN_NOTES.md`: host RAM, image pins, ladder step, pass/fail, wall time
- Soft-skip under `EDGEQUAKE_REQUIRE_POSTGRES_TESTS=1` is a hard fail

## Claim promotion rules

| Claim | Requires |
|-------|----------|
| “50k vectors proven” | Proven row (already green) |
| “100k vectors supported” | L1 green on pg18 release |
| “500k vectors supported” | L2 green |
| “1M entities / workspace” | L3 green **and** graph/entity count measured (not vectors alone) |
| “100k documents / workspace” | L3-scale chunks **and** documented chunks/doc assumption, or dedicated doc-count soak |
