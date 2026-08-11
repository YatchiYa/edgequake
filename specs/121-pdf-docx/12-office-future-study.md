# 12 — Office Future Study (DOCX / Excel)

> **Status:** Non-goal for SPEC-121 v1 product matrix.  
> **Purpose:** Capture how EdgeQuake *could* add Word/Excel later without violating DRY/SOLID or forking ingest.

## First principle

```ascii
  Office bytes
       │
       ▼
  Converter adapter (DOCX | XLSX → Markdown / text)
       │
       ▼
  Existing text admit + KG pipeline
       │
       ▼
  Same chunk / entity / query path as .md uploads
```

Do **not** invent a parallel Office KG pipeline. PDF already established the pattern: convert → durable text/MD → Insert.

## Why not v1

| Reason | Detail |
|--------|--------|
| Product lock | SPEC-121 matrix excludes Office |
| Complexity | ZIP/OOXML security (zip bombs, macros, external relationships) |
| Quality | Tables, track changes, embedded images need policy |
| Ops | Extra crate weight / native deps vs current Docker image |
| Support | #370 closed by honesty + PDF reliability, not feature sprawl |

## Candidate libraries (2026 research)

| Option | Formats | Pros | Cons | Refs |
|--------|---------|------|------|------|
| **undocx** | DOCX → MD | Rust, RAG-oriented, tables/footnotes, `convert_bytes` | Young crate; DOCX-only | [crates.io/undocx](https://crates.io/crates/undocx), [GitHub](https://github.com/KimSeogyu/undocx) |
| **libreoffice-pure** | DOCX/XLSX/PPTX/… | Broad matrix, pure Rust claim, CLI+lib | Not full LibreOffice parity; larger surface | [crates.io/libreoffice-pure](https://crates.io/crates/libreoffice-pure) |
| **anydoc** | Many Office + PDF | Broad GFM conversion, multi-lang bindings | Another PDF path (overlap with pdfium); eval quality variance | [docs.rs/anydoc](https://docs.rs/crate/anydoc/latest) |
| Pandoc / soffice sidecar | Many | Battle-tested | Heavy container; process isolation cost | classic ops pattern |

**Recommended default if Office is funded:** start with **DOCX-only** via `undocx` (or equivalent) as a thin adapter behind `trait OfficeToMarkdown`, skip images for v1 RAG (`skip_images`), then evaluate XLSX separately (sheet → MD tables is a different UX).

## Security checklist (must pass before enable)

| ID | Risk | Mitigation |
|----|------|------------|
| OF-01 | Zip bomb | Max uncompressed size + entry count |
| OF-02 | Billion laughs / entity expansion | Hard XML limits |
| OF-03 | Macro-enabled docs | Reject `.docm` / sniff; no VBA exec |
| OF-04 | External relationships | Block network fetches during convert |
| OF-05 | Embedded binaries | Default skip images; optional quarantine store |
| OF-06 | Path traversal in package names | Sanitize entry paths |

## Suggested architecture (future SPEC)

```ascii
  POST /documents/upload  (extend whitelist)  OR  POST /documents/office
       │
       ▼
  OfficeAdmission
       │  validate magic (PK zip + [Content_Types].xml)
       │  size ceiling
       ▼
  OfficeConvert task (like PdfProcessing)
       │  undocx::convert_bytes
       ▼
  Persist markdown + optional original blob
       ▼
  Enqueue Insert (existing)
```

SOLID: `OfficeConvert` implements same “produce Markdown” port as PDF convert. DRY: one Insert path.

## Excel-specific notes

- XLSX → Markdown tables loses formulas, charts, multiple sheets semantics.
- Product choice required: first sheet only vs all sheets as `## SheetName` sections vs CSV-per-sheet.
- Prefer asking users to export CSV for analytics-shaped data (already API-capable).

## Work estimate (order-of-magnitude)

| Slice | Effort |
|-------|--------|
| Spike undocx in crate + security harness | S |
| Admission + convert job + UI Accept | M |
| E2E corpus (tables, lists, track changes) | M |
| XLSX | M–L (product decisions dominate) |

## Exit criteria for a future “SPEC-12x Office”

1. Security harness green (OF-01..06).  
2. Convert quality bar on golden corpus.  
3. FE/BE/FAQ matrix updated in one PR.  
4. No regression on PDF/text/image paths.

## Cross-refs

- Product non-goal: [00-why.md](00-why.md), [README.md](README.md)
- Architecture reuse: [04-target-architecture.md](04-target-architecture.md)
- Laws: LAW-121-8 in [01-first-principles.md](01-first-principles.md)
