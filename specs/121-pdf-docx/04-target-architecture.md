# 04 — Target Architecture

## Product format matrix (normative)

| Format | Product | Admission | Downstream |
|--------|---------|-----------|------------|
| `.txt` | Supported | `/documents` or `/documents/upload` | Text ingest |
| `.md` | Supported | Same | Text ingest (`source_type: markdown`) |
| `.json` | Supported | Same | Text ingest |
| `.pdf` | Supported | **Only** `/documents/pdf` | Convert → MD → ingest |
| `.png/.jpg/.jpeg/.gif/.webp` | Supported | `/documents/upload` | VLM → text/MD → ingest |
| `.docx` / `.xlsx` / `.xls` | **Not supported** | Reject | — |
| `.csv/.html/.xml/.yaml` | API-only (unchanged) | `/documents/upload` | Text ingest; not WebUI dropzone |

## Target flow

```ascii
                 ┌─────────────────────────────┐
                 │  FormatPolicy SSOT          │
                 │  (extensions + messages)    │
                 └─────────────┬───────────────┘
           ┌───────────────────┼───────────────────┐
           ▼                   ▼                   ▼
        WebUI Accept      API validators      FAQ/OpenAPI
           │                   │
           ▼                   ▼
     classifyUploadFile   route by class
           │
     ┌─────┴──────┬────────────┐
     ▼            ▼            ▼
  TextAdmit    ImageAdmit   PdfAdmit
     │            │            │
     └──────┬─────┴──────┬─────┘
            ▼            ▼
         Insert KG    PdfConvert → Insert KG
```

## SOLID boundaries

| Principle | Application |
|-----------|-------------|
| SRP | `FormatPolicy` = names/messages only; handlers admit; converters convert |
| OCP | New format = new adapter + policy row; do not fork ingest |
| LSP | All converters produce Markdown/text consumable by existing pipeline |
| ISP | PDF clients use PDF API; text clients do not import pdfium types |
| DIP | Pipeline depends on text/MD trait, not Office crates |

## DRY

- One policy table → FE + BE + docs + tests (LAW-121-4).
- PDF convert remains the only binary→MD path in v1.
- Future Office (SPEC follow-up) plugs in as **another converter adapter**, then calls the same text admit (LAW-121-8). See [12-office-future-study.md](12-office-future-study.md).

## PDF reliability checklist (target ops)

```ascii
  [ ] Binary starts; prime_pdfium Ok (or explicit skip with known risk)
  [ ] PDFIUM_AUTO_CACHE_DIR writable by runtime user
  [ ] EDGEQUAKE_MAX_UPLOAD_BYTES == proxy client_max_body_size (≥)
  [ ] Workspace header/UUID present on PDF requests
  [ ] Vision/Ollama (or OpenAI) reachable from container
  [ ] UI distinguishes: admitted | converting | failed(convert) | completed
```

## Error taxonomy (target)

| Code / class | User meaning | HTTP / UI |
|--------------|--------------|-----------|
| `UNSUPPORTED_FORMAT` | Not in product matrix | 400 / toast |
| `FILE_TOO_LARGE` | Over product ceiling | 400/413 / toast |
| `INVALID_PDF` | Magic/empty/corrupt | 400 |
| `WORKSPACE_REQUIRED` | Missing workspace | 400 |
| `PDF_CONVERSION_FAILED` | Convert/vision failed after admit | doc status Failed |
| `PROXY_BODY_LIMIT` | Infra 413 | ops runbook |

## Non-goals in this architecture

- Embedding LibreOffice in the API container for v1
- Accepting DOCX on dropzone “for convenience”
- Unifying PDF into `/documents/upload` without a dedicated convert stage

## Cross-refs

- UX copy: [06-ux-ui-spec.md](06-ux-ui-spec.md)
- Implementation: [07-implementation-plan.md](07-implementation-plan.md)
