# SPEC-074 RUN_NOTES — DiskANN query_rescore recipe + retract P0

- Date: 2026-07-18
- Recipe: `diskann.query_search_list_size=400` **and** `diskann.query_rescore=200` (list/2)
- Helper: `edgequake_storage::diskann_optin_recipe_statements()` / `diskann_query_tuning_statements`
- Silent flip: **Forbidden** (ops/harness `SET LOCAL` only; not boot default)
- Full-gate @150k: green in [SPEC-072](../../../072-diskann-recall-pareto/e2e/artifacts/RUN_NOTES.md) (harness used rescore=list/2)
- Smoke: `make diskann-rescore-smoke` @2000 rows — **full_green** (recall@20=1.0, single p95≈4 ms, stress@16 ok); DiskANN Index Scan confirmed

## Smoke cell

| Cell | Result |
|------|--------|
| q_list=400 q_rescore=200 | recall=1.0 · single_p95≈4 ms · stress_p95≈15 ms · full_green |

## Retract / denorm

- Checklist: [`../../001-retract-checklist.md`](../../001-retract-checklist.md)
- Test: `cargo test -p edgequake-storage --test e2e_spec074_retract_and_denorm`

## Official guidance

pgvectorscale: tune `query_rescore` for accuracy (default **50** is too low with list=400). SPEC-074 makes both GUCs explicit in SSOT.

## Do not

- Run DiskANN @150k with default list=100 / rescore=50
- Enable vectorscale/DiskANN silently on existing DBs
