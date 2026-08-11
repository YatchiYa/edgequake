# Lens 005 — Front Designer

## Stake

Visual affordances must match the product matrix. Showing a DOCX glyph as a first-class upload target implies support that does not exist.

## Guidelines

```ascii
  Dropzone chrome
  ┌──────────────────────────────────────────┐
  │  Drop files here                         │
  │  Supported: TXT, MD, JSON, PDF, images   │
  │  Not supported: DOCX, Excel              │
  └──────────────────────────────────────────┘
```

1. Keep accept list and helper text identical (LAW-121-4).
2. If document table renders a `docx` icon for historical/imported titles, pair with badge “unsupported source” only if such rows can exist; prefer not inventing DOCX rows.
3. PDF rows may show convert progress chrome (existing SPEC-038 patterns) — preserve, do not flatten to generic “upload”.

## Anti-patterns

- Purple gradient “AI upload” marketing chrome that lists “Any document”
- Pill clouds of every extension including Office

## Cross-refs

- UX: [004-ux-ui.md](004-ux-ui.md)
- FE code: `document-dropzone.tsx`, `use-document-dropzone.ts`
