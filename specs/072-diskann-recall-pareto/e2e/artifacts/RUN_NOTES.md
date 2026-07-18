# SPEC-072 RUN_NOTES — DiskANN recall Pareto @150k

- Date: 2026-07-18
- Profile: `pg18-vectorscale` (pgvectorscale 0.9.0)
- Shape: dedicated `*_ws_*` + `USING diskann` (unfiltered ORDER BY — single-WS corpus)
- Recall ref: high DiskANN `query_search_list_size=1600` (ANN-relative; documented)
- Promote SSOT: **YES** (opt-in DiskANN; not silent default)

## Decision

- `green_150k=true promote_ssot=true best=build=default_sbq query_grid_green`
- Full-gate green @150k with **`diskann.query_search_list_size ≥ 400`** (rescore ≥ 200)
- Rebuild arm **not needed** (default SBQ build cleared query grid)
- Spot: 100k green @q≥400; **250k green @q=800** (q=400 recall 0.94)

## Query grid @150k (default build nn=50 sls=100)

| q_list | q_rescore | recall@20 | single p95 | stress p95 @16 | full_green |
|--------|-----------|-----------|------------|----------------|------------|
| 100 | 50 | 0.65 | ~3.8 ms | ~17 ms | no |
| 200 | 100 | 0.97 | ~5.1 ms | ~13 ms | no |
| **400** | **200** | **1.00** | ~6.2 ms | ~13 ms | **yes** |
| **800** | **400** | **1.00** | ~9.0 ms | ~16 ms | **yes** |

## Honesty / product mapping

- **Wave-2 shared+partial** remains the **default** supported 100k path (no silent DiskANN flip).
- **Opt-in DiskANN recipe** (pg18-vectorscale + dedicated table + `query_search_list_size≥400`) raises the dedicated concurrent floor: `highest_green_N=150000` for that recipe.
- SPEC-070 wall was recall@20≈0.98 with default q_list=100; raising search_list clears the gate.
- Filtered JSONB WHERE can force Seq+Sort — measure path uses unfiltered ORDER BY on dedicated single-WS tables (post-filter not required for this shape).

## Artifacts

- `eq-diskann-pareto-pg18-vectorscale.jsonl`
- `eq-diskann-pareto-pg18-vectorscale-cargo.log`
- `PARETO_SUMMARY.md`
