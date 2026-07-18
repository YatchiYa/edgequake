# SPEC-070 RUN_NOTES — DiskANN / pgvectorscale study

- Date: 2026-07-18
- Profile: `pg18-vectorscale` (pgvector 0.8.5 + AGE 1.8.0 + **pgvectorscale 0.9.0**)
- Image: `edgequake-postgres:pg18-vectorscale` (`make postgres-image-build-pg18-vectorscale`)
- Primary shape: dedicated `*_ws_*` + `USING diskann (embedding vector_cosine_ops)` vs HNSW halfvec control
- Gate: single Q1-d (&lt;500 ms) ∧ recall@20≥0.99 ∧ concurrent abs @clients=16
- Promote SSOT: **NO**

## Decision

- `green_150k_diskann=false any_diskann_full_green=true diskann_extension_ok=true smoke=false promote_ssot=false`
- Full-gate green only at **100k DiskANN dedicated** (not a product-default flip; Wave-2 shared+partial stays supported 100k).
- At **150k**, DiskANN concurrent abs is green (~17 ms) but **recall@20 ≈ 0.98** fails the 0.99 gate → no floor raise.
- At **250k**, DiskANN abs stays green; recall cliffs to ~0.45.

## Arms (clients=16)

| rows | arm | single p95 | stress p95 | recall@20 | full_green |
|------|-----|------------|------------|-----------|------------|
| 100k | hnsw_dedicated | ~163 ms | ~3486 ms | 1.00 | no |
| 100k | diskann_dedicated | ~2 ms | ~20 ms | 1.00 | **yes** |
| 150k | hnsw_dedicated | ~214 ms | ~5676 ms | 1.00 | no |
| 150k | diskann_dedicated | ~2 ms | ~17 ms | **0.98** | no |
| 250k | hnsw_dedicated | ~299 ms | ~7968 ms | 1.00 | no |
| 250k | diskann_dedicated | ~2 ms | ~17 ms | 0.45 | no |

## Honesty

- Wave-2 shared+partial remains supported **100k** (`highest_green_N=100k`, `first_fail_N=250k`).
- No silent DiskANN / vectorscale default on existing DBs.
- DiskANN **did** unlock dedicated concurrent vs HNSW @100k–250k abs, but **did not** clear the 150k full gate (recall).
- Opt-in recipe (study): `EQ_POSTGRES_PROFILE=pg18-vectorscale` + `CREATE EXTENSION vectorscale` + `USING diskann` — document only; not promoted floor.

## Artifacts

- `eq-diskann-battle-pg18-vectorscale.jsonl`
- `eq-diskann-battle-pg18-vectorscale-cargo.log`
- `DISKANN_SUMMARY.md`
