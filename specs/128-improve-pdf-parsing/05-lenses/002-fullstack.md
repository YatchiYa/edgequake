# Lens 002 — Full Stack Developer

## Stake

One prune function, one layout DTO, one overlay component. Ingest, include-from-pdf, and parse API must not fork behavior.

## As-is hotspots

| Layer | File | Change |
|-------|------|--------|
| Convert | `edgequake-pdf/src/backend/vision.rs` | prune `figure_map`; persist layout before bbox drop |
| Filter | `figure_filter.rs` | concurrency; new kinds |
| Wire | `document_assets.rs` + `pdf_processing.rs` | set `figure_filter_provider` from vision LLM |
| Include | `include_pdf_assets.rs` | same prune helper |
| pdf2md | `geometry.rs` / `object_cluster.rs` | image area + aspect |
| Storage | new repo methods | pages + regions |
| API | new handler | GET pages / layout; OpenAPI + codegen |
| FE | `pdf-viewer.tsx` + `pdf-page-overlay.tsx` | measured overlay |
| DRY | `documents.ts` vs `document-assets.ts` | one rewrite helper |

## SOLID

- **S:** Filter ≠ persist ≠ overlay ≠ taxonomy map.
- **O:** `PageLayoutExtractor` + `NoopLayoutExtractor`.
- **L:** Any `LLMProvider` for L3; any extractor for L2.
- **I:** Overlay consumes layout DTO only, not mm-asset BYTEA.
- **D:** API does not import `ort`; pdf2md owns inference.

## Wiring WP-1

```ascii
  Vision LLM resolved for Pass-A
       │
       ├─ extract_figures true
       ├─ EDGEQUAKE_FIGURE_FILTER not 0
       └─ cfg.figure_filter_provider = Some(Arc::clone(vision_provider))
```

Do not construct a second provider. Fail-open if clone/name missing.

## API shape (fullstack)

- List pages: counts only (LAW-128-12).
- Layout: current page on overlay toggle / page change (React Query keyed by `{doc, page}`).
- 404 page out of range; 200 + `layout_status=skipped` when L2 off but L0/L1 boxes exist.
- Codegen: `make codegen-openapi-refresh` + contract test.

## Tests this lens owns

- Unit: prune helper 3 keep / 2 discard.
- Contract: ingest with mock provider prunes markdown hrefs.
- Include-from-pdf: discarded not reintroduced.
- Playwright: overlay toggle, zoom alignment, chip filter.

## Cross-refs

- Architecture: [../04-target-architecture.md](../04-target-architecture.md)
- Plan: [../07-implementation-plan.md](../07-implementation-plan.md)
- Test: [../08-test-protocol.md](../08-test-protocol.md)
