# SPEC-077 — Binary quantize + rerank bake-off (B2)

**Status:** Complete (smoke)  
**Depends on:** SPEC-073 §006 (B2 / r5), SPEC-075 (filtered recall), SPEC-076 (exact reorder pattern)  
**Goal:** Measure official pgvector **binary_quantize → Hamming ANN → exact halfvec/vector reorder** vs Wave-2 halfvec HNSW — **no silent flip, no floor raise**.

## Locked decisions

| Decision | Choice |
|----------|--------|
| Default path | Wave-2 shared+partial @100k unchanged |
| Binary + rerank | **Opt-in study / harness only** — not product default |
| Promote metric | **Filtered** recall@20 (SPEC-075) |
| Floors | No raise unless full gate green (out of scope for smoke) |
| Silent flip | Forbidden |

## Pack

| Doc | Content |
|-----|---------|
| [`001-first-principles.md`](001-first-principles.md) | Binary → Hamming candidates → exact reorder |
| [`e2e/artifacts/RUN_NOTES.md`](e2e/artifacts/RUN_NOTES.md) | Bake-off archive |

## Commands

```bash
# Pure SQL/contract (no DB)
cargo test -p edgequake-storage --features postgres --test contract_spec077_binary_quantize

# Bake-off gate (ephemeral PG; smoke N)
make binary-quantize-bakeoff
# EQ_BQ_ROWS=5000 make binary-quantize-bakeoff

make product-limits-check
```

## Checklist

- [x] Pack + first principles
- [x] SQL helpers + contract
- [x] `make binary-quantize-bakeoff` + artifacts
- [x] SSOT tip; SPEC-073 B2 linked; floors unchanged

## Out of scope

Filtered-DiskANN labels (A6), Matryoshka (A5), schema unify (C5), raising Wave-2/DiskANN floors.
