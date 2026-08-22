# Lens 003 — Database Expert

## Stake

Modality and confidence must be **queryable typed fields**, not buried in
`documents.metadata` JSON blobs (SPEC-091).

## Grain

Follow SPEC-128: `document_pages` 1:N regions. Add:

| Column | Nullability | Notes |
|--------|-------------|-------|
| `page_modality` | NOT NULL default `'print'` | Backfill existing rows `print` |
| `transcription_confidence` | NULL | Unknown ≠ 0 |
| `vision_profile` | NULL | Audit: which profile rendered |

## RLS / tenancy

Same policies as `document_pages` today — no new cross-tenant surface.

## Migration rules

- Expand-contract: add columns nullable → backfill → set defaults/NOT NULL as needed.
- Do not store full VLM transcripts twice; MD remains document markdown SSOT.
- Do not put gold fixture binaries in DB migrations.

## Indexes

Optional: `(document_id, page_modality)` only if product filters by modality later.
v1: no new hot index required.

## Cross-refs

- Architecture: [../04-target-architecture.md](../04-target-architecture.md)
- SPEC-128 page grain: [../../128-improve-pdf-parsing/01-first-principles.md](../../128-improve-pdf-parsing/01-first-principles.md)
- SPEC-091: [../../091-simplify-data-layer/](../../091-simplify-data-layer/)
- SOTA: [../12-sota-assessment.md](../12-sota-assessment.md)
