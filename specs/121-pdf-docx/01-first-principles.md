# 01 — First Principles

## Axioms

1. A document format is a **product contract**, not an accidental whitelist drift between FE and BE.
2. Admission path must match content class: text → JSON body or text multipart; binary PDF → PDF endpoint; images → image multipart; Office → none (v1).
3. **Upload success** means the server accepted bytes and created a durable job/row. **Ingest success** means convert (if any) + KG pipeline completed. These are different states.
4. Unsupported formats must **fail closed** with explicit language — never hang, never look like a transient network bug.
5. Observability beats silence: proxy 413, missing workspace, bad magic, pdfium prime fail, and vision timeout must be distinguishable.
6. DRY: one format matrix feeds UI copy, API errors, FAQ, and tests.
7. SOLID: converters (pdfium/vision; future undocx) are adapters; the ingest pipeline consumes Markdown/text only.

## Laws

| Law | Statement |
|-----|-----------|
| **LAW-121-1** | One format → one admission path (no PDF on text whitelist; no DOCX masquerade) |
| **LAW-121-2** | Unsupported formats fail closed with product-honest messaging |
| **LAW-121-3** | Convert/vision failure ≠ “unsupported format” and ≠ silent drop |
| **LAW-121-4** | Format matrix SSOT: FE accept list, BE validators, FAQ, OpenAPI agree |
| **LAW-121-5** | PDF Docker must prime pdfium with writable cache (`PDFIUM_AUTO_CACHE_DIR`) |
| **LAW-121-6** | Size limits are layered (FE, Axum body, reverse proxy) — all must be ≥ product ceiling |
| **LAW-121-7** | Prove with e2e: positive formats + negative Office + wrong-endpoint PDF |
| **LAW-121-8** | Office (DOCX/XLSX) is non-goal v1; any future path converts to Markdown then reuses text ingest |

## Causal diagram (Five WHYs for #370)

```ascii
  WHY “PDF and DOCX not uploading”?
    → Reporter observes only JSON success
  WHY does JSON succeed?
    → Small application/json POST /documents (text path)
  WHY does DOCX fail?
    → Not in FE Accept / BE ALLOWED_EXTENSIONS (by design)
  WHY might PDF fail while JSON works?
    → Different transport (multipart) + different deps (pdfium, vision, workspace)
  WHY does that feel like “upload broken”?
    → UX collapses admit vs convert; docs still say DOCX Planned
```

## Normative admission policy

```ascii
  classify(file):
    pdf?     → POST /documents/pdf      + magic %PDF-
    image?   → POST /documents/upload   + image extensions
    text?    → POST /documents | upload + ALLOWED_EXTENSIONS
    office?  → 400 / toast UNSUPPORTED  (no parser)
    other?   → 400 / toast UNSUPPORTED
```

## Cross-refs

- Matrix authority: [02-cross-ref-matrix.md](02-cross-ref-matrix.md)
- Target: [04-target-architecture.md](04-target-architecture.md)
