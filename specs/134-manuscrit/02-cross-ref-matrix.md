# 02 — Cross-ref matrix

| Ref | Role for SPEC-134 |
|-----|-------------------|
| SPEC-015 / 015V | Vision Pass A/B, prompt overrides, extract gates |
| SPEC-038 | Large PDF adaptive DPI; Auto EdgeParse fast-path (must not steal manuscript) |
| SPEC-047 | RAG vision prompt English Acc pin — conflict to resolve for MS |
| SPEC-049 | Figure filter cascade; display ≠ index; signature discard |
| SPEC-091 | Typed sidecars / page grain — where modality lives |
| SPEC-096 | Extraction language — Pass-A MS must not fight NL language |
| SPEC-128 | PDF vision + layout overlay SSOT; OCR lens; no AGPL weights posture |
| SPEC-133 | `->` in entity names after diagram extract — sibling, do not fork |
| SPEC-057 / 045 | Timeouts / failure_class honesty under higher DPI |
| AGENTS.md | Env table documentation after implement |
| `.env.example` | Operator knobs |
| `edgequake-pdf2md` | pdfium render + VLM convert |

```ascii
  SPEC-038 adaptive DPI ──► print path (keep)
         │
         └─ SPEC-134 ManuscriptProfile DPI floor (override when MS)
                │
  SPEC-047 EN Acc prompt ──► print Pass-A (keep)
         │
         └─ SPEC-134 MS prompt (fidelity, source language)
                │
  SPEC-049 / 128 figure filter ──► print discard signature
         │
         └─ SPEC-134 MS asset policy (LAW-134-14)
                │
  SPEC-133 delimiter ──► KG persist after arrow-heavy MS diagrams
                │
  SPEC-096 language ──► do not force EN paraphrase on MS Pass-A
```

## Doc ↔ code anchors

| Concern | Path |
|---------|------|
| Pass-A RAG prompt SSOT | `edgequake/crates/edgequake-pdf/src/vision_prompts.rs` |
| Figure filter discard kinds | same + `figure_filter.rs` |
| Vision convert / max pixels | `edgequake-pdf/src/backend/vision.rs` |
| Page PNG re-render | `edgequake-pdf/src/page_assets.rs` |
| Backend resolve / Auto | `edgequake-pdf/src/backend/mod.rs` |
| Vision→EdgeParse fallback | `edgequake-pdf/src/fallback.rs` |
| Adaptive DPI / concurrency | `edgequake-api/src/processor/pdf_processing.rs` |
| Vision env models | `edgequake-api/src/vision_env.rs` |
| PDF upload / parse options | `edgequake-api/src/handlers/pdf_upload/`, `handlers/parse/` |
| Multimodal Pass-B | `edgequake-api/src/services/multimodal/` |
| Page layout persist | SPEC-128 `document_pages` / `page_layout_regions` |
| Markdown assemble | `edgequake-pdf/src/vision_markdown.rs` |

## Related specs (read, do not fork)

| Spec | Borrow |
|------|--------|
| [128](../128-improve-pdf-parsing/) | Laws, OCR lens, overlay, no AGPL |
| [133](../133-kv-error/) | Arrow delimiter after MS diagrams |
| [049](../049-improve-figure-extraction/) | Filter ontology |
| [038](../038-ingestion-large-pdf/) | Cost / DPI tradeoffs |
| [096](../096-multi-language-extraction/) | Language fidelity |
| [015-vision-parser](../015-vision-parser/) | Pass A/B contract |

## Law ↔ peer map

| LAW-134                                   | Peer                                       |
| -------------------------------------------| --------------------------------------------|
| LAW-134-1 page-as-unit                    | SPEC-128 OCR lens; SPEC-049 modality split |
| LAW-134-16 graphic-as-unit                | SPEC-049 chart/figure cascade; suppress child crops |
| LAW-134-2 fidelity                        | SPEC-096; anti-SPEC-047 EN pin for MS      |
| LAW-134-3 render                          | SPEC-038 adaptive DPI                      |
| LAW-134-4 display≠index                   | SPEC-128 LAW-128-2                         |
| LAW-134-6 implicit structure + safe names | SPEC-133                                   |
| LAW-134-9 no HTR binary                   | SPEC-128 LAW-128-5                         |
| LAW-134-12 skip EdgeParse                 | SPEC-038 Auto                              |
| LAW-134-20 full-page raster VLM input     | pdf2md render; assemble; empty-page retry  |

## Cross-refs

- Laws: [01-first-principles.md](01-first-principles.md)
- Code as-is: [03-code-as-is.md](03-code-as-is.md)
- Plan: [07-implementation-plan.md](07-implementation-plan.md)
- SOTA: [12-sota-assessment.md](12-sota-assessment.md)
