# Lens 002 — Full Stack Developer

## Stake

Three admission surfaces already exist (JSON text, multipart upload, PDF). Drift between FE Accept, BE whitelist, and docs creates false bugs. Fixes must be policy + tests, not a fourth ad-hoc list.

## As-is hotspots

| Layer | File | Risk |
|-------|------|------|
| FE | `use-document-dropzone.ts` | Accept vs toast strings can diverge from FAQ |
| FE | `perform-file-upload.ts` | Correct PDF routing — must stay SSOT |
| BE | `file_validation.rs` | No `.pdf` — correct for this route; wrong if docs point PDF here |
| BE | `handlers/pdf_upload/*` | Magic, workspace, size, convert enqueue |
| Ops | Dockerfile / compose | pdfium cache + Ollama host |

## Target work (DRY/SOLID)

```ascii
  FormatPolicy (names + user messages)
       │
       ├─ FE Accept builder
       ├─ BE validate_extension / image / pdf gate messages
       └─ docs snippet generator or checked table in FAQ
```

1. Extract shared “supported formats” string for toasts and API `BadRequest` (or keep parallel constants with a contract test that diffs them).
2. Add e2e: DOCX/XLSX reject; PDF on `/upload` → clear 400; PDF on `/pdf` → 202/admit.
3. Map convert failures to distinct UI status (do not reuse unsupported-format toast).

## Anti-patterns

- Adding `.docx` to Accept “temporarily” without a converter
- Parsing DOCX as UTF-8 text
- Putting PDF back into `ALLOWED_EXTENSIONS` without convert stage

## Cross-refs

- Code as-is: [../03-code-as-is.md](../03-code-as-is.md)
- Tests: [../08-test-protocol.md](../08-test-protocol.md)
