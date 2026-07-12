# 003 — Implementation plan

## P0 — Object-first regions (this delivery)

1. Add `pipeline/visual/*` with geometry + object clustering + caption labeling + crop render.
2. `extract_visual_regions_from_bytes/path` public API; bump pdf2md **0.9.5**.
3. Classify clusters:
   - Contains Image or Form → prefer `Figure`
   - Dominated by thin horizontal rules → `Table`
   - Caption label may refine kind when attached
4. Wire EdgeQuake `region_assets` to visual extract (DRY facade).
5. Keep chart residual after fig/table; area invariant unchanged.
6. Tests: unit geometry + fixture e2e (vector fig, table, embedded image, empty page).

## P1 — StructTree L0

1. ✅ Expose `StructTreeProposer` via Pdfium FFI (`PdfiumStructTreeProposer`,
   public `get_handle_from_page` — no private `page_handle`).
2. ✅ Tagged fixture: Figure element → L0 proposal before L1 (`tagged_figure_sample.pdf`, E13).
3. ✅ Telemetry: `pages_with_struct_tree` + `l0_regions` in extract log.

## P2 — Layout model (optional)

Pinned ONNX DocLayout-class detector; only if L0+L1 empty; CI mAP gate.

## P3 — Vector-preserving crop

MuPDF clip / Form-only render research; retire keyword chart gate as proposal source.

## P4 — Non-flaky fidelity lift (see 005)

Ordered levers that raise recall/precision **without** English detectors:

1. **P1a** ✅ Fix same-page ImageXObject skip → IoU merge with Form placements.  
2. **P1b** ✅ Placement-first L1 (Pdfium page-space Image/Form seeds; path residual outside seeds).  
3. **P1c** ✅ Form-first precision: suppress path-only subsets (`refine_proposals`).  
4. **P2** ✅ Residual **ink mask** propose (`chart_residual_candidate_pages` + page-PNG ink prefilter); `text_suggests_chart` specialize-only.  
5. **P3** ✅ Real StructTree L0 + tagged fixture (G4 / E13).  
6. Optional frozen L2 ONNX; text lattice tables on a separate track.

Process rule: every geometry constant change requires corpus Δ (G6–G8 in 005).

## Rollout

1. Land pdf2md **0.9.7** + EdgeQuake workspace bump.  
2. Re-run include-from-pdf / vision path on agentic_hardware smoke.  
3. Update specs/CHANGELOG.
4. Execute 005 phases with e2e gates on `specs/048-improve-ux/e2e/`.
