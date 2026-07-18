# SPEC-074 — Storage reliability + DiskANN precision (P0)

**Status:** Complete (2026-07-18)  
**Depends on:** SPEC-058/059 (retract), SPEC-072 (DiskANN @150k), SPEC-073 §006 (research P0)  
**Goal:** Harden document retract completeness and productize opt-in DiskANN **`query_rescore`** alongside `query_search_list_size≥400` — no silent flips, no floor raise.

## Locked decisions

| Decision | Choice |
|----------|--------|
| Default path | Wave-2 @100k unchanged |
| DiskANN | Opt-in; list≥400 **and** rescore (tip: rescore = list/2, e.g. 200 @ 400) |
| Silent flip | Forbidden |
| Floors | Unchanged unless later full-gate says otherwise |

## Pack

| Doc / path | Content |
|------------|---------|
| [`001-retract-checklist.md`](001-retract-checklist.md) | Operator + eng retract completeness |
| [`e2e/artifacts/RUN_NOTES.md`](e2e/artifacts/RUN_NOTES.md) | DiskANN rescore recipe evidence |
| Storage tests | `e2e_spec074_*` / denorm + delete_by_document |
| Helper | `diskann_query_tuning_statements` (ops/harness; not boot default) |

## Commands

```bash
# Retract / denorm (memory always; Postgres when DATABASE_URL set)
cargo test -p edgequake-storage --test e2e_spec074_retract_and_denorm -- --nocapture

# DiskANN rescore smoke (pg18-vectorscale ephemeral)
EQ_DISKANN_SMOKE=1 make diskann-rescore-smoke

make product-limits-check
```

## Checklist

- [x] Pack + retract checklist
- [x] Retract / denorm e2e (`e2e_spec074_retract_and_denorm`)
- [x] DiskANN rescore helper + RUN_NOTES (`diskann_optin_recipe_statements`)
- [x] SSOT: product-limits + FAQ + product_limits_check (`query_rescore`)

## Out of scope

Binary quantize, Filtered-DiskANN labels, unified `document_chunks`, iterative_scan-vs-partial bake-off, Louvain/Mix floors.
