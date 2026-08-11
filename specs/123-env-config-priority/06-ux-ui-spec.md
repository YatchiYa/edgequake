# 06 — UX / UI Spec

## Surfaces

| Surface | Control | Copy rule |
|---------|---------|-----------|
| Workspace settings | PDF Parser select | Options: Server Default (leaf), Vision, EdgeParse, Auto |
| Tenant settings (if exposed) | Same | Tenant-level default |
| Documents upload | Parser for this upload | Workspace Default (leaf); Vision; EdgeParse; Auto |
| Large PDF admission | Parser choice | Override applies to **large files only** |
| Document detail | Extraction method badge | Effective method; Auto note if rewrote |

## Resolved chip

```ascii
  ┌──────────────────────────────┐
  │ PDF Parser                   │
  │ [ Server Default (Vision) ▼] │  Resolves to Vision
  └──────────────────────────────┘
```

When choice is Auto:

```ascii
  │ [ Auto ▼ ]                   │  May use EdgeParse for born-digital PDFs
```

## i18n keys (add/update)

- `settings.pdfParser.auto`
- `settings.pdfParser.resolvesToAuto`
- `documents.upload.pdfParserAuto`
- `documents.upload.largePdfAdmission.appliesToLargeOnly`

## Cross-ref

- Laws: [01-first-principles.md](01-first-principles.md) LAW-123-3,6
- Lenses: [05-lenses/004-ux-ui.md](05-lenses/004-ux-ui.md)
