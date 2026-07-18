# SPEC-070 — DiskANN / pgvectorscale study

**Status:** Measured — **honest wall** (no SSOT floor raise)  
**Opened by:** SPEC-069 dedicated concurrent wall  
**Depends on:** SPEC-069, SPEC-068 mid-scale wall, SPEC-071 Wave-2 turnkey  

## Commands

```bash
make postgres-image-build-pg18-vectorscale
make diskann-battle                          # default rows 100k,150k,250k
EQ_DISKANN_SMOKE=1 make diskann-battle       # extension smoke only
```

## Locked decisions

| Decision | Choice |
|----------|--------|
| Extension | pgvectorscale **0.9.0** on pg18 (`CREATE EXTENSION vectorscale CASCADE`) |
| Image | Opt-in `EQ_POSTGRES_PROFILE=pg18-vectorscale` — **not** product default |
| Primary battle | Dedicated `*_ws_*` + `USING diskann` vs HNSW halfvec control |
| Promote | Full gate only: single Q1-d ∧ recall@20≥0.99 ∧ concurrent abs @clients=16 @**150k** |
| Silent flip | Forbidden |

## Result (2026-07-18)

| N | HNSW clients=16 | DiskANN clients=16 | DiskANN recall@20 | Full green |
|---|-----------------|--------------------|-------------------|------------|
| 100k | ❌ ~3.5 s p95 | ✅ ~20 ms p95 | 1.00 | DiskANN only |
| 150k | ❌ ~5.7 s p95 | ✅ ~17 ms abs | **0.98** (&lt;0.99) | ❌ |
| 250k | ❌ ~8.0 s p95 | ✅ ~17 ms abs | 0.45 | ❌ |

**Promote SSOT:** **NO** — 150k DiskANN fails recall gate; floors stay `highest_green_N=100k`.  
Wave-2 shared+partial remains the supported 100k product path. DiskANN is opt-in study only (concurrent win on dedicated @100k does not redefine the product default).

Artifacts: [`e2e/artifacts/`](e2e/artifacts/) · [`RUN_NOTES.md`](e2e/artifacts/RUN_NOTES.md)
