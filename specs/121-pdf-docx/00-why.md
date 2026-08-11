# 00 — Why SPEC-121

## Trigger

Partner report on Docker **v0.24.2** ([GitHub #370](https://github.com/raphaelmansuy/edgequake/issues/370)):

> PDF and DOCX documents are not uploading successfully. Only JSON files are being uploaded successfully.

Expected by reporter: PDF, DOCX, and JSON all succeed.

## Product WHY

```ascii
  User expects: “I can put my knowledge into EdgeQuake”
       │
       ▼
  Product truth (locked for SPEC-121):
       │
       ├─ Markdown / plain text / JSON  → must admit reliably
       ├─ Images                        → must admit (VLM path)
       ├─ PDF                           → must admit + convert (vision/pdfium)
       └─ DOCX / Excel                  → must REJECT clearly (not “bug”)
              │
              ▼
  Without clarity:
       • Partners file bugs against correct rejects (DOCX)
       • Real PDF path failures hide behind “same as DOCX”
       • FAQ “DOCX Planned” contradicts roadmap → trust erosion
```

## Two different truths inside one issue

| Claim in #370 | Product truth | Engineering action |
|---------------|---------------|--------------------|
| JSON uploads | Supported (text path) | Keep green; use as control in repro |
| DOCX does not upload | **Correct** — out of matrix | Messaging + tests that reject is intentional |
| PDF does not upload | **Should work** — separate pipeline | Reproduce; triage multipart / pdfium / vision / proxy |

## Gaps

| Approach / artifact | Gap |
|---------------------|-----|
| FAQ “DOCX Planned” | Over-promises vs product lock (no Office in v1) |
| FAQ format table | Omits images + JSON; HTML marked Planned while API allows it |
| Tutorials / curl examples | Some show PDF on `/documents/upload` (whitelist rejects `.pdf`) |
| FE vs BE lists | FE: TXT/MD/JSON/PDF/images; BE text upload: no PDF; PDF has own route |
| Error UX | Convert/vision failures often read as “upload failed” |
| Proxy body size | JSON tiny; PDF multipart large → 413 asymmetry ([classic nginx](https://stackoverflow.com/questions/24306335/413-request-entity-too-large-file-upload-issue)) |

## Success

1. One published format matrix: **TXT, MD, JSON, PDF, PNG/JPG/GIF/WEBP** supported; **DOCX/XLSX not supported**.
2. Dropzone, API errors, FAQ, and OpenAPI say the same thing (LAW-121-4).
3. PDF admit on `/documents/pdf` is proven in Docker; convert failures are labeled as convert, not “unsupported format”.
4. #370 comment documents investigation; issue closes only when acceptance green (or confirmed env with runbook).
5. Office ingest documented only as future study ([12-office-future-study.md](12-office-future-study.md)).

## Cross-refs

- Laws: [01-first-principles.md](01-first-principles.md)
- Code: [03-code-as-is.md](03-code-as-is.md)
- Repro: [10-reproduction.md](10-reproduction.md)
