# Lens 008 — System Engineer

## Stake

ONNX + pdfium + Axum must coexist without deadlocks, AGPL contamination, or surprise downloads in GHCR.

## Build / ship

```ascii
  pdf2md feature layout-onnx
       → dep ort = "=2.0.0-rc.13"
       → CPU EP default
       → optional CUDA later (document only)

  EdgeQuake Docker
       → do NOT copy AGPL .pt/.onnx from DocLayout-YOLO
       → optional COPY models/pp-doclayout-v3.onnx + sha256 file
       → env LAYOUT_ONNX_MODEL_PATH
       → if missing: layout_status=skipped, ingest continues
```

Dev: `[patch.crates-io] edgequake-pdf2md = { path = "../../edgequake-pdf2md" }` (mirror llm). Merge/CI: published crate pin.

## Runtime isolation (LAW-128-16)

- Pdfium: existing `spawn_blocking` / singleton — **do not** run ONNX inside the pdfium lock.
- `ort::Session::run` takes `&mut self` — **one session per worker thread**, or mutex per process (document latency). Prefer session pool size = `EDGEQUAKE_PDF_CONCURRENCY` cap.
- Page routing skips figure PNG I/O when no Image/Form and empty L1.

## Resources

| Knob | Guidance |
|------|----------|
| Layout imgsz | Match model (800 PP-V3) |
| VLM jobs | existing `EDGEQUAKE_PDF_VISION_JOBS` |
| Filter concurrency | 4 |
| Max VLM/page | 12 |

Layout on CPU ~tens of ms to low hundreds per page; must not block the HTTP thread.

## Integrity

`layout_onnx_model_sha256` required when path set. Mismatch → `layout_status=failed`, fail-open (no ingest abort), log error.

## Observability / ops

Counters + spans from [04-target-architecture.md](../04-target-architecture.md). Health: optional `layout_onnx: { enabled, model_sha256_ok }` on `/health` — only if cheap (file exists); do not load the session on health.

## License review gate

CI comment or `make spec128-proof` asserts default model card license Apache-2.0 in the pinned URL list in [13-layout-taxonomy.md](../13-layout-taxonomy.md). DocLayout-YOLO path is documented as unsupported in product images.

## Cross-refs

- Plan WP-6/7: [../07-implementation-plan.md](../07-implementation-plan.md)
- Risk: [../11-honest-assessment.md](../11-honest-assessment.md)
