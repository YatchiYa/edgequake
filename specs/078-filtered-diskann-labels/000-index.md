# SPEC-078 — Filtered-DiskANN labels bake-off (A6) + Phase-3 assessment

**Status:** Complete (smoke)  
**Depends on:** SPEC-073 §006 (A6 / r6), SPEC-070/072/074 (DiskANN opt-in), SPEC-075 (filtered recall), SPEC-077 (B2 smoke)  
**Goal:** Measure pgvectorscale **Filtered-DiskANN** (`labels smallint[]` in index + `&&`) vs Wave-2 and vs post-filter DiskANN — **no silent flip, no floor raise**.

## Locked decisions

| Decision | Choice |
|----------|--------|
| Default path | Wave-2 shared+partial @100k unchanged |
| Dedicated DiskANN | Supported opt-in @150k unchanged (embedding-only / dedicated tables) |
| Filtered-DiskANN labels | **Opt-in study / harness only** — not product default |
| Promote metric | **Filtered** recall@20 (SPEC-075) |
| Floors | No raise unless full gate green (out of scope for smoke) |
| Silent flip | Forbidden (no product `labels` migration) |

## Pack

| Doc | Content |
|-----|---------|
| [`001-first-principles.md`](001-first-principles.md) | UUID→`smallint` map; labels in graph walk |
| [`002-phase3-assessment.md`](002-phase3-assessment.md) | Phase-3 status after 077/078 |
| [`e2e/artifacts/RUN_NOTES.md`](e2e/artifacts/RUN_NOTES.md) | Bake-off archive |

## Commands

```bash
# Pure SQL/contract (no DB)
cargo test -p edgequake-storage --features postgres --test contract_spec078_filtered_diskann_labels

# Bake-off gate (ephemeral pg18-vectorscale; smoke N)
make filtered-diskann-labels-bakeoff
# EQ_FDL_ROWS=5000 make filtered-diskann-labels-bakeoff

make product-limits-check
```

## Checklist

- [x] Pack + first principles + Phase-3 assessment
- [x] SQL helpers + contract (default OFF)
- [x] `make filtered-diskann-labels-bakeoff` + artifacts
- [x] SSOT tip; SPEC-073 A6 linked; floors unchanged

## Out of scope

Product `labels` migration, wiring into `query_filtered`, Matryoshka (A5), C5 serving view, raising Wave-2/DiskANN floors.
