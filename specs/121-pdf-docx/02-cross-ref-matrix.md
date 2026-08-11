# 02 — Cross-ref Matrix

## Claim → Authority

| Claim | Authority |
|-------|-----------|
| JSON uploads; PDF/DOCX do not (partner) | [#370](https://github.com/raphaelmansuy/edgequake/issues/370) |
| Product supports TXT/MD/JSON/PDF/images; not DOCX/XLSX | SPEC-121 product lock ([00-why.md](00-why.md)) |
| FE accept list omits DOCX | `edgequake_webui/src/hooks/use-document-dropzone.ts` |
| FE routes PDF vs text vs image | `edgequake_webui/src/lib/upload/file-kind.ts`, `perform-file-upload.ts` |
| BE text whitelist has no pdf/docx | `edgequake-api/src/file_validation.rs` `ALLOWED_EXTENSIONS` |
| PDF endpoint + magic | `handlers/pdf_upload/upload.rs`; `validate_pdf_data` |
| pdfium prime fail-closed | `edgequake/src/main.rs` `prime_pdfium`; SPEC-095 |
| Docker pdfium cache writable | `edgequake/docker/Dockerfile`; `PDFIUM_AUTO_CACHE_DIR`; #100 |
| Body limit default 50 MiB | `EDGEQUAKE_MAX_UPLOAD_BYTES`; Axum `DefaultBodyLimit` |
| FAQ lists DOCX Planned (drift) | `docs/faq.md` Features table |
| Injection rejects .docx/.pdf | `edgequake-api/tests/e2e_injection.rs` |
| Proxy 413 asymmetry | nginx `client_max_body_size` class of bugs |
| Future DOCX→MD crates | [undocx](https://crates.io/crates/undocx), [libreoffice-pure](https://crates.io/crates/libreoffice-pure), [anydoc](https://docs.rs/crate/anydoc/latest) |
| Laws | LAW-121-1..8 ([01-first-principles.md](01-first-principles.md)) |

## Code SSOT (as-is → target)

| Concern | Path |
|---------|------|
| FE dropzone Accept + toasts | `edgequake_webui/src/hooks/use-document-dropzone.ts` |
| FE upload router | `edgequake_webui/src/lib/upload/perform-file-upload.ts` |
| FE kind classify | `edgequake_webui/src/lib/upload/file-kind.ts` |
| FE size SSOT | `edgequake_webui/src/lib/api/upload-limits.ts` |
| BE text/image validation | `edgequake/crates/edgequake-api/src/file_validation.rs` |
| Multipart file upload | `.../handlers/documents/upload/file_upload.rs` |
| PDF upload | `.../handlers/pdf_upload/upload.rs` |
| PDF routes | `edgequake-api/src/routes.rs` `/documents/pdf*` |
| PDF storage validate | `edgequake-storage/.../pdf_storage.rs` |
| Multimodal admission | `edgequake-api/src/services/multimodal_admission.rs` |
| Docker image | `edgequake/docker/Dockerfile` |
| Format FAQ | `docs/faq.md` |
| PDF tutorial | `docs/tutorials/pdf-ingestion.md` |
| Upload quick ref | `docs/api-reference/document-upload-quick-reference.md` |

## Related specs / issues

| Spec / Issue | Relationship |
|--------------|--------------|
| GH #370 | This mission |
| SPEC-013 | PDF upload / progress / cancel |
| SPEC-024 | Async / batch file upload |
| SPEC-083 | Sanitize + magic mismatch |
| SPEC-095 | pdfium prime + atomic cache |
| GH #100 | pdfium cache permission in containers |
| SPEC-038 | Upload progress UX |
| LightRAG parity / SPEC-026 P-06 | Historical DOCX Phase-3 — superseded as product non-goal for v1 |

## DRY rule

One **format policy** table drives:

1. FE `Accept` + i18n toast strings  
2. BE `ALLOWED_*` + PDF route gate  
3. FAQ / quick-reference / OpenAPI descriptions  
4. Negative e2e (DOCX/XLSX/wrong-endpoint PDF)

Do not invent a fourth list in marketing copy. Office converters (future) must emit Markdown into the existing text ingest path — never a parallel KG pipeline (LAW-121-8).
