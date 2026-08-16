# 10 — Edge Cases

Every row: mitigation + test id from [08-test-protocol.md](08-test-protocol.md).

| ID | Case | Mitigation | Test |
|----|------|------------|------|
| E-128-01 | Filter classifies but assemble still lists discards | Rebuild figure_map (WP-0) | G-prune |
| E-128-02 | Filter never wired | WP-1 default-on | T-wire-1 |
| E-128-03 | Filter throws | Fail-open keep all; log | T-prune-2 |
| E-128-04 | include-from-pdf resurrects discards | Shared prune / don’t rewrite discarded | T-prune-3 |
| E-128-05 | Tiny real chart dropped by area | Floor 0.008; G6 lock | T-geo-2 + G6 |
| E-128-06 | Banner/logo 12:1 aspect | MAX_ASPECT | T-geo-1 |
| E-128-07 | Overlay vs RAG split | Persist abandon; prune assets | G-layout + G-industrial |
| E-128-08 | L2 down / missing onnx | skipped/failed; L0/L1 persist | T-onnx-1 |
| E-128-09 | sha256 mismatch | failed, fail-open | T-onnx-2 |
| E-128-10 | L2 box vs L0 same visual | Keep L0 | T-l0-1 |
| E-128-11 | Rotation 90/180/270 | Store rotation; transform in bbox_norm | T-norm-1 + fixture |
| E-128-12 | CropBox ≠ MediaBox | Store cropbox_pdf; overlay uses displayed box | unit |
| E-128-13 | Scanned page no glyphs | L1 images + L2 on raster; Pass-A OCR | e2e scan fixture if present |
| E-128-14 | Text-only page | No fig PNGs; paragraph/column overlay | WP-5 + T-page |
| E-128-15 | Empty / 0-page / corrupt PDF | existing empty-ok; no page rows | E11 SPEC-049 |
| E-128-16 | Near-full Form 80% | MAX_AREA_FRAC | G3 |
| E-128-17 | 500-page PDF | Lazy GET per page | contract list vs detail |
| E-128-18 | Reprocess stale overlay | Delete pages then rewrite | integration |
| E-128-19 | Document delete | CASCADE | G-cascade |
| E-128-20 | Cross-workspace GET | RLS | G-rls |
| E-128-21 | Zoom + width-fit | Measure onRenderSuccess | G-overlay |
| E-128-22 | Overlay 0 regions | Empty copy, no fake boxes | UX spec |
| E-128-23 | Dual-pane markdown click | asset_path scroll | Playwright |
| E-128-24 | pdf.js TextLayer vs boxes | Overlay above canvas, pointer-events on boxes only | FE |
| E-128-25 | ort `&mut Session` races | One session per worker | system lens |
| E-128-26 | ONNX holds pdfium lock | Forbidden; spawn_blocking split | code review + test comment |
| E-128-27 | AGPL weights in image | Not copied; LAW-128-5 | B10 |
| E-128-28 | Unpinned download | Forbidden in prod | config |
| E-128-29 | Two-column paper | derived columns | T-col-1 |
| E-128-30 | Vertical / RTL | class other; overlay still axis-aligned AABB | honest |
| E-128-31 | Table text-native vs visual | modality split; overlay table without PNG | SPEC-049 E15 |
| E-128-32 | Pass-2 unused at assemble | v1 OK; do not block prune | honest |
| E-128-33 | chunks.page_start SQL null | Overlay independent | non-goal |
| E-128-34 | document_id null during process | Wait until linked (default) | DB lens |
| E-128-35 | Chip state vs other doc | sessionStorage chips global OK; layout fetch per doc/page | FE |
| E-128-36 | Missing testids | Add pdf-viewer / overlay | Playwright |
| E-128-37 | Duplicate URL rewrite | DRY one helper | fullstack |
| E-128-38 | Health loads 130MB onnx | Health must not load session | system |
| E-128-39 | FORM layout miss | FORM_LAYOUT_EXEMPT | vision |
| E-128-40 | Caption invents crop | still banned | G5 / E12 |

## Cross-refs

- Laws: [01-first-principles.md](01-first-principles.md)
- Test: [08-test-protocol.md](08-test-protocol.md)
- Honest: [11-honest-assessment.md](11-honest-assessment.md)
