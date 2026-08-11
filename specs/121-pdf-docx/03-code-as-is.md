# 03 — Code As-Is

## Endpoint matrix

| Endpoint | Content-Type | Accepts | Rejects |
|----------|--------------|---------|---------|
| `POST /api/v1/documents` | `application/json` | Text body (any string; title may be `*.json`) | N/A extension gate |
| `POST /api/v1/documents/upload` | multipart | `ALLOWED_EXTENSIONS` + images | `.pdf`, `.docx`, binaries |
| `POST /api/v1/documents/upload/batch` | multipart | Same via `resolve_upload_content` | Same |
| `POST /api/v1/documents/pdf` (+ `/batch`) | multipart | PDF magic `%PDF-` | Non-PDF bytes |
| Injection file upload | multipart | `txt, md, csv, json` | `.pdf`, `.docx` |

## Frontend routing

```ascii
  classifyUploadFile(file)
       │
       ├─ isPdfUploadFile     → uploadPdfDocument  → /documents/pdf
       ├─ isImageUploadFile   → uploadFile         → /documents/upload
       └─ else (text/json/md) → file.text()+JSON   → /documents
```

Sources:

- Accept map: `use-document-dropzone.ts` — `.txt .md .json .pdf` + images; **no docx**
- Router: `perform-file-upload.ts`
- Kind: `file-kind.ts`

Max size: 50 MiB (`upload-limits.ts` ↔ `EDGEQUAKE_MAX_UPLOAD_BYTES`).

## Backend whitelist (text multipart)

```rust
// file_validation.rs
pub const ALLOWED_EXTENSIONS: [&str; 9] = [
    "txt", "md", "json", "csv", "html", "htm", "xml", "yaml", "yml",
];
pub const ALLOWED_IMAGE_EXTENSIONS: [&str; 5] =
    ["png", "jpg", "jpeg", "gif", "webp"];
```

Note: CSV/HTML/XML/YAML are **API-capable** but not on the WebUI dropzone Accept list.

## PDF pipeline (supported)

```ascii
  POST /documents/pdf
       │
       ├─ sanitize filename (SPEC-083)
       ├─ magic matches .pdf
       ├─ validate_pdf_data (non-empty, ≤ max, %PDF-)
       ├─ workspace UUID required
       ▼
  Admit PdfProcessing (convert)
       │
       ├─ pdfium render / edgeparse
       ├─ vision LLM (when enabled)
       ▼
  Durable markdown → Insert ingest task
```

Failure codes users may see: `Invalid PDF`, magic mismatch, `Workspace ID required`, `PDF_CONVERSION_FAILED`, `PDF_PAGE_FAILED`, stuck `Converting` if vision host unreachable.

## Docker constraints

- Image embeds pdfium via `edgequake-pdf2md` / pdfium-auto.
- Runtime: `/tmp/edgequake-pdfium-cache` mode 1777; env `PDFIUM_AUTO_CACHE_DIR`.
- Startup: `prime_pdfium()` unless `EDGEQUAKE_SKIP_PDFIUM_PRIME=1`.
- Compose often sets `OLLAMA_HOST=http://host.docker.internal:11434` — vision must reach host.

## Docs drift (as-is)

| Doc | Drift |
|-----|-------|
| `docs/faq.md` | DOCX “Planned”; HTML “Planned”; no JSON/images row |
| Some upload curls | PDF against `/documents/upload` → 400 unsupported |

## Why #370 splits

```ascii
  JSON  → text JSON path → small body → no pdfium → SUCCESS
  DOCX  → not accepted   → toast / 400 → “not uploading” (expected)
  PDF   → multipart path → pdfium+vision+workspace → FAIL if env broken
```

## Cross-refs

- Target: [04-target-architecture.md](04-target-architecture.md)
- Repro steps: [10-reproduction.md](10-reproduction.md)
