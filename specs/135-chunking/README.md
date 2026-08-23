# SPEC-135 — PDF ingest pack-to-budget

> **Mission:** Stop PDF ingest from underfilling the workspace token budget.
> Pack converted markdown (and figures) to tiktoken size; index each figure
> **once**; treat page as **attribution** (span allowed); persist `page_start` /
> `page_end` on relational columns.
>
> **Trigger:** 23 Aug 2026 — workspace Fixed 1200/100; 16-page technical PDF;
> display markdown ~26k tiktoken; live `chunk_count=70`; token p50 **230**
> (19% fill). Not a size-config miss. SPEC-125 packed `.md`; PDF still emits
> per page × per atomic region, then appends duplicate MM sidecars.

## Short verdict

| Layer | Finding |
|-------|---------|
| Symptom | Operator pins Acc-fair 1200/100; PDF becomes ~3× as many extract jobs as a LightRAG **F** window on the same text |
| Classification | **Strategy + index-once bug**, not “chunk size too small” |
| Root cause | `PageAwareChunking` wraps Recursive (word-count); atomic regions flush as their own chunks; `enrich_processed_text_with_mm_chunks` re-appends VLM already inlined in Pass-A markdown; page columns never bound on insert |
| Fix posture | P0 MM-once + skip comment-only; P1 PDF inner = SPEC-125 packer (tiktoken); P2 undersize remainder may span pages; persist page columns; fill observability |
| Non-goals | Late chunking; Anthropic LLM contextual prefixes; semantic-V; changing Acc **R** `recursive_token_len` or **F** on non-PDF; auto-rebuild |

```ascii
  PDF → Pass-A markdown + page markers + inline VLM
       │
       ├─ TODAY: PageAware(Recursive words)
       │         × atomic emit × MM sidecar append
       │         → N=70, p50=230, page_start column NULL
       │
       └─ TARGET: pack to tiktoken budget
                  page is attribution (span OK)
                  MM indexed once
                  → N~24–32 on trigger class, p50 ≥ ~800 @ 1200
```

## Document map

```ascii
 00-why
   → 01-first-principles (LAW-135-1..12)
   → 02-cross-ref-matrix
   → 03-code-as-is
   → 04-target-architecture
   → 05-lenses/ (PO, fullstack, DB, UX, AI, RAG)
   → 06-ux-ui-spec
   → 07-implementation-plan
   → 08-test-protocol
   → 09-acceptance
   → 10-edge-cases
   → 11-research-evidence-aug-2026
   → 12-honest-assessment
   → fixtures/ (synthetic only — no live paper)
```

## Status board

| ID | Item | Status |
|----|------|--------|
| D1 | Doc pack (00–12 + 6 lenses + fixtures) | This tree |
| I1 | P0: comment-only skip + MM inline dedupe | Done (v0.26.0) |
| I2 | P1: Pdf inner = markdown packer (tiktoken) | Done (v0.26.0) |
| I3 | P2: cross-page remainder pack + span | Done (v0.26.0) |
| I4 | Persist `page_start`/`page_end` columns | Done (v0.26.0) |
| I5 | `ingest.chunking` `fill_p50` + mm sidecar flag | Done (v0.26.0) |
| I6 | UX span badge `p.N–M` + deeplink start page | Done (v0.26.0) |
| T1 | Unfakable U-135-* + E2E-135-01 + Playwright + Acc R | Done (v0.26.0) |

## Config (v1)

```ascii
  Document chunk_options  >  Workspace ChunkingPolicy  >  Fleet env  >  Default
       │
       └─ Tenant: ABSENT (SPEC-123 gap; not in v1)

  Pdf strategy auto for .pdf / source_type=pdf / page markers (SPEC-032)
  Packing default ON;  EDGEQUAKE_PDF_PACK=0             → pre-135 Recursive inner
  Cross-page default ON; EDGEQUAKE_PDF_CROSS_PAGE_PACK=0 → hard page emit (P1 only)
  MM chunks default ON;  EDGEQUAKE_MM_CHUNKS=0           → no sidecar append at all
  Inline dedupe default ON when sidecars would duplicate Pass-A VLM
```

Wizard **Match LightRAG (Acc fair)** remains SPEC-116 size/overlap **1200/100**.
This spec makes product PDF **fill** that budget. It does **not** become byte-identical
LightRAG **F** (structure-aware pack, not a raw token sliding window).

## Related

- [SPEC-026](../026-egdequake-vs-lightrag/) — F/R/P/V strategies; Acc recursive cascade
- [SPEC-033](../033-page-lineage/) — page citation; **amend** `page_start == page_end`
- [SPEC-047](../047-rag-evaluation/) — atomic MM / charts; sidecar append Acc path
- [SPEC-091](../091-simplify-data-layer/) — relational chunk authority
- [SPEC-116](../116-adaptive-chunking/) — workspace size/overlap (orthogonal)
- [SPEC-124](../124-langfuse-support/) — `ingest.chunking` observation I/O
- [SPEC-125](../125-better-chunking/) — markdown packer SSOT (reuse; E10/E30 reversed here)
- [SPEC-134](../134-manuscrit/) — `grounding:low` stripped before chunk (unchanged)

## Non-goals (v1)

- Jina late-chunking / full-doc embed-then-pool
- Per-chunk LLM “situate this chunk” (`EDGEQUAKE_CONTEXTUAL_CHUNK` stays opt-in)
- `ChunkStrategy::Semantic`
- Changing Acc Recursive `recursive_token_len` or TokenBased **F** on plain text
- Auto-rebuild KG on workspace save (LAW-116-4)
- Tenant-level chunking
- Rewriting Pass-A / Pass-B vision prompts
