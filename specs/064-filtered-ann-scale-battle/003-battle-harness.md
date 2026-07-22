# SPEC-064 — Battle harness

## Entry points

| Surface | Path |
|---------|------|
| Test | `edgequake/crates/edgequake-storage/tests/e2e_spec064_ann_scale_battle.rs` |
| Runner | [`e2e/run_ann_scale_battle.sh`](e2e/run_ann_scale_battle.sh) |
| Make | `make ann-scale-battle` |

## Environment

| Var | Default | Meaning |
|-----|---------|---------|
| `EDGEQUAKE_PERF_SCALE` | **required `large`** | 100k @1536 (L1) |
| `EDGEQUAKE_PERF_RELEASE` | `1` (runner) | `--release` |
| `EDGEQUAKE_BATTLE_ARMS` | all four | comma list: `full_default,halfvec_default,halfvec_partial_ws,guc_grid` |
| `EDGEQUAKE_HNSW_PARTIAL_BY_WORKSPACE` | unset | opt-in partial DDL path |
| `EDGEQUAKE_HNSW_EF_SEARCH` | unset → `max(40, 4×top_k)` | Wave3 |
| `EDGEQUAKE_HNSW_MAX_SCAN_TUPLES` | `20000` | Wave3 |
| `EDGEQUAKE_HNSW_SCAN_MEM_MULTIPLIER` | unset | Wave3 |

## Artifacts

Runner writes:

- `e2e/artifacts/eq-battle-pg18.jsonl`
- `e2e/artifacts/eq-battle-pg18-cargo.log`
- `e2e/artifacts/WAVE0_EXPLAIN.md` (extracted from `battle_full_default_explain`)

## Code levers

- `PgVectorStorage::with_storage_mode(VectorStorageMode::Half|Full)`
- `ensure_partial_hnsw_for_workspace` / `drop_global_ann_index`
- Search GUCs in [`search_tuning.rs`](../../edgequake/crates/edgequake-storage/src/adapters/postgres/vector/search_tuning.rs)
