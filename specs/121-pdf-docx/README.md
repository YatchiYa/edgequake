# SPEC-121 — Upload Format Matrix & PDF Reliability

> **Mission:** Lock the product document-format matrix (Markdown / text / image / PDF supported; DOCX / Excel out of scope), make unsupported formats fail closed with honest UX, and harden PDF admit+convert so Docker deployments do not look like “PDF upload broken” while JSON succeeds.  
> **Trigger:** [GitHub #370](https://github.com/raphaelmansuy/edgequake/issues/370) — “PDF and DOCX Documents Not Uploading” (v0.24.2 Docker).

## Short verdict

| Layer              | Finding                                                                            |
| --------------------| ------------------------------------------------------------------------------------|
| Symptom            | Reporter: JSON uploads; PDF and DOCX do not                                        |
| DOCX               | **Not a product format** — FE dropzone + BE whitelist reject by design             |
| PDF                | **Supported** on a **different path** (`POST /documents/pdf`) than JSON            |
| Proximate PDF risk | Multipart / proxy body size / workspace header / pdfium cache / vision unreachable |
| Docs drift         | FAQ corrected (SPEC-121); DOCX/Excel **Not supported**                  |
| Fix v1             | Messaging SSOT + PDF route hints + runbook + unit tests; **no Office** |

```ascii
  UI / API upload
       │
       ├─ .txt .md .json ──► POST /documents            (text admit)
       ├─ image/*        ──► POST /documents/upload     (VLM)
       ├─ .pdf           ──► POST /documents/pdf        (pdfium + vision)
       └─ .docx .xlsx    ──► REJECT (product matrix)
```

## Document map

```ascii
 00-why
   → 01-first-principles (LAW-121-1..8)
   → 02-cross-ref-matrix
   → 03-code-as-is
   → 04-target-architecture
   → 05-lenses/ (PO, fullstack, DB, UX, front, marketing, system)
   → 06-ux-ui-spec
   → 07-implementation-plan
   → 08-test-protocol
   → 09-acceptance
   → 10-reproduction
   → 11-honest-assessment
   → 12-office-future-study (DOCX/XLSX only; non-goal v1)
```

## Status board

| ID | Item | Status |
|----|------|--------|
| D1 | Doc pack | Done |
| R1 | Local reproduction JSON / PDF / DOCX | Done (unit/FE + code; live Docker curl deferred) |
| G1 | GitHub #370 investigation comment | Done ([comment](https://github.com/raphaelmansuy/edgequake/issues/370#issuecomment-5249431519)) |
| I1 | Format-matrix SSOT + FAQ/docs sync (P0) | Done |
| I2 | PDF error taxonomy + Docker runbook (P1) | Done (FAQ runbook + API/FE reject copy) |
| T1–T12 | Contract / e2e / Playwright matrix | Done for T4–T7 unit + FE; T1–T3/T8–T12 via existing + docs |
| A1 | Acceptance | Mostly green — #370 open pending reporter ack |
| H1 | Honest assessment | Done |
| F1 | Office future study | Done |

## Related

- [Issue #370](https://github.com/raphaelmansuy/edgequake/issues/370)
- SPEC-013 PDF upload / progress
- SPEC-024 async / batch upload
- SPEC-083 filename sanitize + magic MIME
- SPEC-095 pdfium prime + cache
- FAQ / tutorials: `docs/faq.md`, `docs/tutorials/pdf-ingestion.md`
- LightRAG parity gap (historical “DOCX Planned”) — superseded for product v1 by this SPEC

## Non-goals (v1)

- Shipping DOCX or Excel (XLSX) ingest
- Changing PDF vision / edgeparse algorithms
- SPEC-120 legacy_vector absorb work
- Expanding UI to CSV/HTML/XML/YAML (API-only text remains as-is; messaging may note API-only)
