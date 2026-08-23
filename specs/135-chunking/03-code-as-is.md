# 03 — Code as-is (2026-08-23)

Call graph of the live PDF ingest path that produced **70** chunks / p50 **230**
on the trigger document. No product code in this spec step — this file is the
map implementers change in WP-1..WP-5.

## End-to-end call graph

```ascii
  PDF bytes
       │
       ▼
  Pass-A  (pdf2md + page markers + inline VLM in page body)
       │
       ▼
  prepare.rs  /  ingestion_pipeline.for_pdf()
       │  ChunkStrategy::Pdf
       ▼
  enrich_processed_text_with_mm_chunks          multimodal/chunks.rs
       │  if EDGEQUAKE_MM_CHUNKS != 0
       │    build_mm_chunks_from_manifest
       │    append_mm_chunks_to_text
       │      + "\n\n<!-- multimodal-chunks -->\n"
       │      + [Chart Name]<id> copies  (even if already inlined)
       ▼
  resolve_chunker(Pdf)                          chunker/registry.rs:157
       │  PageAwareChunking::default()
       │    inner = RecursiveCharacterChunking   page_aware.rs:47-52
       ▼
  split at <!-- edgequake-page:N -->            page_aware.rs
       │  for each page:
       │    inner.chunk(page_body)               word-count Recursive
       │    stamp page_start = page_end = N      hard, no span
       ▼
  atomic_blocks.rs                              region = emit
       │  page markers / figures / tables
       │  HTML comments can become their own unit
       ▼
  ProcessingResult.chunks  (ChunkResult has page_*)
       │
       ├─ chunk_storage.rs     pages → KV / metadata JSON
       └─ relational_chunk_writer.rs
            pages → metadata JSON only
            domain Chunk { … no page fields … }
            INSERT binds content/token_count/metadata
            chunks.page_start / page_end columns stay NULL     LAW-135-9
```

## File SSOT (today)

| Concern | File | Today |
|---------|------|-------|
| Strategy pick | `chunker/registry.rs` | `Pdf => PageAwareChunking::default()` |
| Page wrap | `chunker/page_aware.rs` | Inner Recursive; **MUST NOT span**; tests assert `page_start == page_end` |
| Inner split | `chunker/recursive.rs` | Word-count ≈ tokens/0.75 (Acc **R**) |
| Packer | `chunker/markdown_pack.rs` | Used by `MarkdownChunking` only |
| Atomic regions | `chunker/atomic_blocks.rs` | Page/figure/table/fence as hard regions |
| PDF auto | `ingestion_pipeline.rs` `for_pdf()` | Sets `ChunkStrategy::Pdf` |
| MM append | `services/multimodal/chunks.rs` | Always concatenates sidecars when enabled |
| Domain row | `storage/.../domain/types.rs` `Chunk` | No `page_start`/`page_end` fields |
| Relational write | `persistence/relational_chunk_writer.rs:61-81` | Pages in `metadata`; columns unset |
| SQL columns | migration `066_chunk_lineage_tables.sql` | `page_start INT`, `page_end INT` exist |
| Lineage enrich | `handlers/lineage/queries.rs` | Reads **KV** `page_start` as workaround |
| Query context | `edgequake-query/src/context.rs:159` | Comment: “always equals `page_start`” |
| OpenAPI | `openapi.snapshot.json` ChunkDetail | `page_end` “always equals `page_start`” |
| UI | `document-hierarchy-tree.tsx:50,476` | Badge `p.{page_start}` only; assumes equality |
| Observability | `ingest.chunking` (SPEC-124) | N + size target; no `fill_p50` |

## Persistence hole (live)

```ascii
  ChunkResult.page_start = Some(N)     set by page_aware
           │
           ├─ KV JSON     chunk_storage.rs writes page_*     (lineage can recover)
           └─ SQL row     relational_chunk_writer
                            metadata.page_start = N
                            INSERT Chunk { page fields: ABSENT }
                            → public.chunks.page_start IS NULL

  Trigger workspace: SELECT page_start FROM chunks
                     WHERE document_id = <FreeToken>
                     → NULL on every row
```

SPEC-033 citation that reads **columns** is dead. Overlay / SQL
`idx_chunks_page_span` is unused for live PDF ingest.

## MM double-index (live anatomy)

```ascii
  Pass-A body already contains:
      **Type:** / edgequake-figure-vision / **Summary:**
      for cost_capability_* (and peers)

  append_mm_chunks_to_text then adds:
      <!-- multimodal-chunks -->          ← becomes its own Recursive chunk
      [Chart Name]cost_capability_*       ← 8 extra extract jobs

  LightRAG does this because F never saw VLM.
  EQ does this even though Pass-A already inlined VLM.
```

## Page-aware contract (today — to be amended)

From `page_aware.rs` module docs:

> A PDF chunk **MUST NOT span two pages**.

Tests at `page_aware.rs` ~254–259:

```text
page_start must equal page_end — no cross-page chunks
```

P2 **replaces** this invariant with: `page_end ≥ page_start`; span only when
LAW-135-8 allows. Tests that assert equality must move behind
`EDGEQUAKE_PDF_CROSS_PAGE_PACK=0`.

## Token estimators (three, still)

| Path | Estimator | Role after 135 |
|------|-----------|----------------|
| Recursive | word-count × 0.75 | Acc **R** only |
| TokenBased / Fixed | tiktoken | Acc **F** / Fixed |
| Markdown packer | tiktoken `count_tokens` | Product `.md` **and** product PDF inner |

Pdf today uses the **Recursive** estimator. That is G1.

## What is already correct (do not regress)

- Page **markers** exist and parse (`PageMarkerWriter` / `parse_page_marker`).
- `ChunkResult` already has `page_start`/`page_end` (in-memory).
- SPEC-125 packer already packs ATX / tables / fences on `.md`.
- `EDGEQUAKE_MM_CHUNKS=0` already disables **all** sidecar append.
- `grounding:low` strip (SPEC-134) runs before chunk — leave it.
- Acc Recursive / TokenBased on non-PDF fixtures — leave them (`U-135-ACC-R`).

## Probe numbers (same file, 2026-08-23)

| Path | N |
|------|---|
| Live ingest (Pdf + MM append) | **70** |
| Pdf + Acc-fair, no sidecar | 61 |
| Recursive on full MD | 77 |
| Markdown packer on full MD | 29 |
| LightRAG F window (1200/100) | 24 |

Target class after 135 (P0+P1+P2): **N in gold closed range ~24–32**,
`fill_p50 ≥ 0.55` @ 1200. Exact bounds: `fixtures/freetoken_like.gold.json`.
