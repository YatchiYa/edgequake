# 07 — Implementation Plan

## Principles

- **DRY:** one format matrix → FE Accept, BE validators, FAQ, OpenAPI, tests (LAW-121-4)
- **SOLID:** converters are adapters; ingest consumes Markdown/text only (LAW-121-8)
- **First principles:** unsupported ≠ broken; convert ≠ upload (LAW-121-2, LAW-121-3)
- **Test first:** negative Office + wrong-endpoint PDF before claiming fixed (LAW-121-7)

## Phase A — Truth & messaging (P0)

1. Define canonical matrix table in SPEC (this pack) and sync:
   - [`docs/faq.md`](../../docs/faq.md) — remove DOCX “Planned”; add JSON/images; mark DOCX/Excel **Not supported**
   - Dropzone helper + toast strings (`use-document-dropzone.ts` / i18n)
   - API error strings for unsupported extensions (point PDF callers to `/documents/pdf` when `.pdf` hits text upload)
2. Fix misleading curls in upload quick-reference / tutorials that POST PDF to `/documents/upload`.
3. Contract test or shared constant: FE listed extensions ⊆ product matrix; BE text list + PDF route cover positives.

## Phase B — PDF reliability (P1)

1. Operator runbook in FAQ / ops doc (proxy body size, `PDFIUM_AUTO_CACHE_DIR`, vision host).
2. Ensure health or startup logs make pdfium prime status obvious.
3. UI/API: map `PDF_CONVERSION_FAILED` / vision timeout to convert-failed copy (not unsupported).
4. Verify compose defaults: `EDGEQUAKE_MAX_UPLOAD_BYTES`, cache dir, `OLLAMA_HOST`.

## Phase C — Tests (P2)

See [08-test-protocol.md](08-test-protocol.md) T1–T12. Implement any missing asserts:

1. FE unit: DOCX/XLSX → invalid type
2. API: `.docx` / `.xlsx` on `/documents/upload` → 400
3. API: `.pdf` on `/documents/upload` → 400 with hint toward `/documents/pdf`
4. API: valid PDF on `/documents/pdf` → admit
5. Playwright: dropzone rejects Office; PDF progress path smoke

## Phase D — Docs / GitHub

1. Comment on #370 with matrix + SPEC-121 link + triage questions.
2. Close #370 only when [09-acceptance.md](09-acceptance.md) green.
3. Office work tracked only via [12-office-future-study.md](12-office-future-study.md) (no code in v1).

## Edge-case matrix

| ID | Case | Mitigation | Test |
|----|------|------------|------|
| EC-01 | `.docx` dropped on UI | Toast unsupported | T5 |
| EC-02 | `.docx` multipart API | 400 whitelist | T6 |
| EC-03 | `.xlsx` / `.xls` | Same reject | T7 |
| EC-04 | `.pdf` on `/documents/upload` | 400 + route hint | T4 |
| EC-05 | Valid PDF `/documents/pdf` | Admit 2xx | T3 |
| EC-06 | Non-PDF bytes named `.pdf` | Magic fail | T9 |
| EC-07 | Empty PDF | Invalid PDF | T9 |
| EC-08 | Oversize PDF/text | FE + body limit | T8 |
| EC-09 | Proxy 413 | Runbook; align limits | T8 / ops |
| EC-10 | Missing workspace on PDF | 400 Workspace required | T12 |
| EC-11 | pdfium cache RO | Fail-closed / docs | T10 |
| EC-12 | Vision down after admit | Failed convert status | T11 |
| EC-13 | JSON control upload | Remains green | T1 |
| EC-14 | Image PNG upload | Multipart VLM path | T2 |
| EC-15 | Batch mixed PDF+JSON | Each uses correct route | existing batch + assert |
| EC-16 | Filename path traversal | `sanitize_filename` | SPEC-083 tests |
| EC-17 | ZIP/DOCX renamed `.txt` | UTF-8 / binary reject | file_validation |
| EC-18 | CSV via API (not UI) | Still allowed on upload | optional assert |
| EC-19 | Duplicate PDF | Existing duplicate_of | SPEC-013 |
| EC-20 | Cancel during convert | SPEC-013 cancel | existing |

## Rollout

1. Land SPEC pack + GitHub comment (this mission).
2. Land P0 docs/messaging + missing tests.
3. Land P1 runbook / error taxonomy.
4. Close #370 after acceptance.
5. Office: separate SPEC if/when funded.

## Cross-refs

- Target: [04-target-architecture.md](04-target-architecture.md)
- Tests: [08-test-protocol.md](08-test-protocol.md)
- Acceptance: [09-acceptance.md](09-acceptance.md)
