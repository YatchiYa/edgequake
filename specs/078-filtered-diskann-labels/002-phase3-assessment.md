# SPEC-078 — Phase-3 assessment (after SPEC-077)

SPEC-073 §006 Phase 3 = scale bake-offs (B2, B6, A6). Exit criterion (pre-078): *B2 smoke done; mid-scale/full gate + A6 still open — no silent default.*

## Status board

| Item | Pri | Status | Notes |
|------|-----|--------|-------|
| **B2** binary quantize + rerank | P1 | **Smoke done** [SPEC-077](../077-binary-quantize-bakeoff/000-index.md) | Mid-scale/full gate before any promote |
| **A6** Filtered-DiskANN labels | P2 | **Smoke done** [SPEC-078](000-index.md) | Shared-table DiskANN + `labels &&`; not product default |
| **B6** DiskANN build params | P1 | Deferred | Only if `q_list≥400` still fails at higher N |
| **A5** Matryoshka / `num_dimensions` | P1 | Deferred | Needs prefix-capable embeddings |
| **B3** tiny-slice exact | P0 | Open (light eng) | Not this pack |
| **C5** serving view | P2 | Phase 4 | After scale bake-offs |

## Done before Phase 3

| Phase | Packs |
|-------|-------|
| 0 Reliability (C1–C2) | SPEC-074 |
| 1 Precision knobs (A1–A2, B5) | SPEC-074 / 075 |
| 2 Precision layers (A3–A4) | SPEC-076 |

## Exit criteria (Phase 3)

| Criterion | Target |
|-----------|--------|
| B2 smoke | Done (077) |
| A6 smoke | Done when `make filtered-diskann-labels-bakeoff` green + RUN_NOTES archived |
| Promote floors | **Forbidden** from smoke alone |
| Silent flip | **Forbidden** |
| Mid-scale / full gate | Still open for any promote of B2 or A6 |

## Recommended next after this pack

1. Optional mid-scale re-run (B2 and/or A6) before any SSOT promote  
2. Light eng: B3 tiny-slice exact  
3. Phase 4: C5 serving view (only if retract surfaces decrease without recall loss)

## Anti-patterns (unchanged)

- Promote from unfiltered-only latency demos  
- Silent `labels` / DiskANN / halfvec migration on existing DBs  
- Treat dedicated HNSW as scale unlock (SPEC-069)  
- Raise Wave-2 100k or DiskANN 150k from smoke cells  
