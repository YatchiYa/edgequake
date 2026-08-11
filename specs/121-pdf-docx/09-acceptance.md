# 09 — Acceptance

## Product acceptance

- [x] Published matrix: TXT, MD, JSON, PDF, PNG/JPG/GIF/WEBP **supported**
- [x] DOCX and Excel **not supported**, with explicit UX + FAQ (not “Planned”)
- [x] Partner-facing language matches engineering (LAW-121-4)

## Technical acceptance

- [x] JSON/text control upload succeeds (T1) — covered by existing upload path + FAQ/control; `validate_extension` allows json/md/txt
- [x] Image upload path succeeds (T2) — FE `file-kind` image classify + existing Playwright image-upload (not re-run this turn)
- [x] PDF admits on `/documents/pdf` (T3) — routing SSOT + docs; existing SPEC-013 suite
- [x] PDF on `/documents/upload` returns clear 400 (T4) — `spec121_pdf_on_text_upload_hints_pdf_route` **PASS**
- [x] DOCX/XLSX rejected FE + API (T5–T7) — FE toast/office copy + `spec121_office_extensions_rejected_with_clear_message` **PASS**; FE vitest Office classify
- [x] Oversize / bad magic / missing workspace covered (T8–T9, T12) — existing `file_validation` size/magic tests; workspace fail-closed unchanged
- [x] Convert failure distinguishable from unsupported (T11, LAW-121-3) — FAQ runbook + Office/PDF reject copy ≠ convert codes
- [x] Docker pdfium/cache runbook documented (T10 / ops) — FAQ PDF Docker runbook + compose defaults

## Process acceptance

- [x] SPEC-121 pack complete with cross-refs + ASCII
- [x] [10-reproduction.md](10-reproduction.md) has recorded evidence
- [x] GitHub #370 investigation comment posted with SPEC link
- [ ] #370 closed only after above green **or** labeled environmental with runbook ack from reporter — **follow-up comment posted; leave open for reporter ack**

## Non-acceptance (do not claim done)

- Shipping DOCX/XLSX ingest
- “All formats LightRAG supports”
- Silent ignore of Office drops

## Sign-off

| Role | Sign-off means |
|------|----------------|
| PO | Matrix honesty + PDF usable in supported Docker |
| Full stack | Tests T1–T12 addressed or waived with reason |
| System | Runbook verified once on Docker |

## Cross-refs

- WHY success: [00-why.md](00-why.md)
- Tests: [08-test-protocol.md](08-test-protocol.md)
