# SPEC-049 — Improve figure / table / chart extraction

> **Status:** Active  
> **Date:** 2026-07-12  
> **Depends on:** SPEC-047 multimodal assets, edgequake-pdf2md ≥ 0.9.5  
> **Related:** [pdf-asset-first-principles canvas](../../.cursor/projects/…), ISO 32000-1 §8.8–8.10 / §14.7

## Goal

Extract figure, table, and chart region images from PDFs using a **cascade of
authoritative signals** — not English caption keywords or Pass-A chart word lists
as primary detectors. Keep Pdfium as the single render SSOT. Obey DRY / SOLID.

## Documents

| Doc | Purpose |
|-----|---------|
| [001-first-principles.md](./001-first-principles.md) | Ontology, banned heuristics, invariants |
| [002-architecture.md](./002-architecture.md) | Cascade L0–L3, module boundaries (SOLID) |
| [003-implementation-plan.md](./003-implementation-plan.md) | Phased delivery P0–P4 |
| [004-acceptance-and-tests.md](./004-acceptance-and-tests.md) | Edge-case matrix + CI gates |
| [005-non-flaky-improvement-brainstorm.md](./005-non-flaky-improvement-brainstorm.md) | First-principles levers without flaky heuristics |

## Non-goals (this spec)

- Replacing VLM page→markdown OCR
- Dual render engines as pixel SSOT
- Acc / score heuristics for RAG eval
- Shipping DocLayNet weights in P0/P1 (P2 optional)

## Success snapshot

- Vector Form XObject figures crop without ImageXObject and without inventing paths
- Tables are region crops (≤55% page), never full-page chart dumps
- Caption strings **label** regions; they do not **detect** them
- StructTree (when tagged) overrides object clusters for those elements
- Contract + e2e suites cover the edge-case matrix in 004
