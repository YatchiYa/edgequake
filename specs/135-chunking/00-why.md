# 00 — Why SPEC-135 exists

## The trigger (measured, 2026-08-23)

Workspace **spec132-repro** / **spec132-ws**: Chunking **Fixed 1200 / 100**, embedding
**mistral-embed** (8192 ctx — no 1200 cap). Document
`free_token_2608.16157v1.pdf` completed.

| Fact                                       | Value                                             |
| --------------------------------------------| ---------------------------------------------------|
| Source                                     | 16-page technical PDF, converted markdown on disk |
| Bytes / tiktoken / words                   | 99,211 / **26,412** cl100k / 14,016               |
| Page markers                               | 16 (`<!-- edgequake-page:N -->`)                  |
| Live `documents.chunk_count`               | **70**                                            |
| Token p50 / avg / min / max                | **230** / 440 / 6 / 1717                          |
| Fill vs 1200                               | p50 **19%**                                       |
| LightRAG **F** window on same text         | **24** chunks                                     |
| Markdown packer (SPEC-125) on same text    | **29** chunks                                     |
| Pdf+Acc-fair production probe (no sidecar) | **61** chunks                                     |

The operator did not pick the wrong size. The PDF path **does not pack**.

## Anatomy of the 70

```ascii
  chunks[0..60]   61  page-aware Recursive word-count units
  chunks[61]       1  orphan  <!-- multimodal-chunks -->
  chunks[62..69]   8  [Chart Name] / [Figure Name] sidecar copies
                   ──
                   70
```

Chunks 0–60 already contain Pass-A inlined VLM (`**Type:**`, `edgequake-figure-vision`,
`**Summary:**`). Sidecars 62–69 copy the same assets as `[Chart Name]cost_capability_*`.
That is **double-index**, not “missing figure text.”

LightRAG appends `_build_mm_chunks_from_sidecars` when the **F** token window never
saw VLM text. EdgeQuake already inlined VLM in the markdown. Mechanical copy of
LightRAG’s append is the wrong *intent*.

## Five WHYs

```ascii
  WHY-1  Why 70 chunks on a 26k-token paper at 1200/100?
         → 61 Recursive page-aware units + 1 comment + 8 MM copies.

  WHY-2  Why 61 instead of ~24–32?
         → PageAware hard-splits on page markers, then Recursive
           word-count + atomic-block emit (figure, table, heading)
           flushes small regions instead of packing to budget.

  WHY-3  Why Recursive inside Pdf?
         → SPEC-125 E10/E30 left PDF on Recursive so Acc PDF
           geometry would not drift. Product PDF paid that cost.

  WHY-4  Why orphan comment + 8 sidecars?
         → append_mm_chunks_to_text always concatenates
           <!-- multimodal-chunks --> + [Chart Name] blocks.
           Recursive then extracts the comment as its own unit.

  WHY-5  Why page_start NULL on live rows?
         → page_aware sets ChunkResult.page_start/page_end, but
           relational_chunk_writer puts pages in metadata JSON
           and never binds chunks.page_start / page_end columns.
           SPEC-033 citation cannot fire from the table.
```

## What this is not

```ascii
  NOT a “raise chunk size” ticket     (1200 is already Acc-fair)
  NOT a Pass-A OCR failure            (markdown is complete)
  NOT an embedding-context overflow   (mistral-embed 8192)
  NOT Acc R/F on non-PDF              (those paths stay pinned)
```

## Causal chain

```ascii
  Pass-A markdown
       │  page markers + inline VLM + tables + ATX
       ▼
  enrich_processed_text_with_mm_chunks
       │  ALWAYS append sidecar copies of the same figures
       ▼
  ChunkStrategy::Pdf → PageAware(RecursiveCharacterChunking)
       │  word-count ≈ tokens/0.75
       │  atomic region = emit (not pack-with-neighbor)
       │  page boundary = hard flush
       ▼
  N = pages × regions + comment + sidecars
       │
       ▼
  extract jobs explode; embeddings underfill; citations lose page columns
```

## The one-sentence product claim

**A PDF ingested at workspace 1200/100 must pack converted markdown to that
budget (structure as constraint), index each figure once, and store page
spans on the chunk row — without changing Acc Recursive / TokenBased on
non-PDF text.**

## Cross-refs

- Laws: [01-first-principles.md](01-first-principles.md)
- As-is: [03-code-as-is.md](03-code-as-is.md)
- Acc honesty: [12-honest-assessment.md](12-honest-assessment.md)
