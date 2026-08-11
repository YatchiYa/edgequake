# 06 — UX / UI Spec

## Surfaces

| Surface | Behavior |
|---------|----------|
| Document dropzone | Accept only product formats; list them in chrome |
| Reject toast | Immediate; no upload row for Office |
| Upload progress row | PDF shows convert phase; text/image do not |
| Document status | Failed convert shows conversion message |
| Empty / help | Link to FAQ format matrix |

## Dropzone chrome (copy)

**Supported:** TXT, Markdown, JSON, PDF, PNG, JPG, GIF, WEBP  

**Not supported:** DOCX, Excel (XLSX/XLS) — export to PDF or Markdown instead.

## State machine (PDF)

```ascii
  Selected
     │
     ▼
  Uploading (multipart bytes)
     │
     ├─ 4xx admit error ──► Toast + row error (no convert)
     └─ admitted
            │
            ▼
         Converting
            │
            ├─ success ──► Processing (KG) ──► Completed
            └─ fail    ──► Failed (convert) + Retry CTA
```

## Accessibility

- Announce reject reason via toast + `aria-live`
- Do not rely on color alone for Failed vs Completed
- File input `accept` attribute matches Accept map

## i18n keys (target)

| Key | Purpose |
|-----|---------|
| `upload.supportedFormats` | Dropzone helper |
| `upload.unsupportedFormat` | Toast (include matrix) |
| `upload.fileTooLarge` | Size toast |
| `upload.pdfConvertFailed` | Convert failure |
| `upload.officeNotSupported` | Explicit Office reject |

## Out of scope

- New marketing landing redesign
- Drag-drop redesign beyond copy/accept alignment

## Cross-refs

- Lenses: [05-lenses/004-ux-ui.md](05-lenses/004-ux-ui.md), [05-lenses/005-front-designer.md](05-lenses/005-front-designer.md)
- Acceptance: [09-acceptance.md](09-acceptance.md)
