# 02 — Cross-Reference Matrix

## Spec / doc map

| Spec / Doc | Relationship to SPEC-135 |
|------------|--------------------------|
| [SPEC-026](../026-egdequake-vs-lightrag/) | Acc **R** `recursive_token_len` and **F** token window stay on non-PDF. Product Pdf strategy is **not** Acc R. |
| [SPEC-032](../032-graph/) | Page markers + `PageAwareChunking`. 135 amends “chunk MUST NOT span two pages.” |
| [SPEC-033](../033-page-lineage/) | Citation via `page_start`/`page_end`. **Amend:** `page_end` may exceed `page_start`. Deep-link still `#page={page_start}`. |
| [SPEC-047](../047-rag-evaluation/) | MM sidecars when VLM is **absent** from the token stream. 135: skip sidecar when Pass-A already inlined the same asset. |
| [SPEC-091](../091-simplify-data-layer/) | Relational `chunks` is authority. Domain `Chunk` today has **no** page fields — persistence hole (G4). |
| [SPEC-116](../116-adaptive-chunking/) | Size/overlap policy (1200/100). Orthogonal. Wizard “Match LightRAG” pins **size**, not LightRAG **F**. |
| [SPEC-123](../123-env-config-priority/) | Tenant layer **not** added; documented gap. |
| [SPEC-124](../124-langfuse-support/) | `ingest.chunking` I/O. Add `fill_p50` + `mm_sidecar_appended`. |
| [SPEC-125](../125-better-chunking/) | Packer SSOT (`markdown_pack.rs`). **E10/E30 reversed:** PDF inner = packer. |
| [SPEC-128](../128-improve-pdf-parsing/) | Overlay / page columns: writers fill JSONB only — 135 closes that for new ingest. |
| [SPEC-134](../134-manuscrit/) | `grounding:low` stripped **before** pack. Unchanged. |

## Violation / gap register

| ID | Gap | Law | Fix |
|----|-----|-----|-----|
| G1 | Pdf inner = Recursive word-count | LAW-135-3,4 | Inner = `markdown_pack.rs`; tiktoken |
| G2 | Atomic region = emit (flush-before-neighbor) | LAW-135-1,2 | Pack-with-neighbor until budget |
| G3 | Hard page emit even when remainder + next page fit | LAW-135-7,8 | P2 soft page + span |
| G4 | `chunks.page_start`/`page_end` columns NULL; pages only in JSON metadata | LAW-135-9 | Bind columns (extend domain `Chunk` if needed) |
| G5 | Inline VLM + `[Chart Name]` sidecar double-index | LAW-135-5 | Dedupe in `enrich_processed_text_with_mm_chunks` |
| G6 | Comment-only extract units (`<!-- multimodal-chunks -->`, lone page marker) | LAW-135-6 | Skip in atomic walk / packer |
| G7 | SPEC-125 E10/E30 left PDF on Recursive | LAW-135-3 | This spec reverses them |
| G8 | SPEC-033 / OpenAPI: `page_end` always equals `page_start` | LAW-135-8 | Amend copy + UI badge `p.N–M` |
| G9 | `ingest.chunking` records N, not fill | LAW-135-10 | `fill_p50` = p50/budget |
| G10 | Acc PDF geometry assumed stable | LAW-135-12 | Honest re-score or `PDF_PACK=0` |
| G11 | Tenant not in chunking cascade | SPEC-123 | Honest non-goal v1 |

## ASCII dependency

```ascii
  SPEC-116  HOW LARGE (1200/100 policy)
       │
       ├─ SPEC-125  WHERE TO CUT  .md files  (packer SSOT)
       │
       └─ SPEC-135  WHERE TO CUT  PDF-converted markdown
              │
              ├─ reuse markdown_pack.rs (LAW-135-3)
              ├─ MM once (SPEC-047 intent, not mechanical append)
              ├─ page span (amends SPEC-032 / SPEC-033 equality)
              ├─ persist columns (SPEC-091 authority)
              └─ fill_p50 on ingest.chunking (SPEC-124)
```

## What SPEC-125 already forbids (still true)

| 125 ID | Still true under 135 |
|--------|----------------------|
| E10 “page markers → Pdf, packer not used” | **False after 135.** Packer **is** the Pdf inner. |
| E30 “PDF converted MD → Pdf unchanged” | **False after 135.** Pdf geometry changes. |
| LAW-125-5 table header repeat | Still true (packer reuse) |
| LAW-125-8 fences / MM interiors atomic | Still true |
| Acc R `recursive_token_len` | Unchanged (LAW-135-12) |

## Code SSOT (target)

| Concern | Path |
|---------|------|
| Packer (reuse) | `edgequake-pipeline/src/chunker/markdown_pack.rs` |
| Pdf strategy | `chunker/registry.rs` `ChunkStrategy::Pdf` |
| Page wrapper | `chunker/page_aware.rs` inner + P2 span pass |
| Atomic / comments | `chunker/atomic_blocks.rs` |
| MM append / dedupe | `edgequake-api/.../multimodal/chunks.rs` |
| Domain + insert | `edgequake-storage/.../domain/types.rs` `Chunk`; postgres bind |
| Writer | `persistence/relational_chunk_writer.rs` |
| Lineage | `handlers/lineage/` + OpenAPI `ChunkDetail` |
| UI | `document-hierarchy-tree.tsx` |
| Observability | `ingest.chunking` / `langfuse_meta.rs` |
| Kill switches | `.env.example` `EDGEQUAKE_PDF_PACK`, `EDGEQUAKE_PDF_CROSS_PAGE_PACK` |

## DRY rule

Packer math lives **only** in `markdown_pack.rs`. Pdf must not fork a second greedy packer. Recursive Acc merge stays in `recursive.rs`. UI never reimplements packing or page-span math.

## Amendment list (other specs)

After 135 lands, patch copy in:

- `specs/125-better-chunking/10-edge-cases.md` E10, E30
- `specs/125-better-chunking/09-acceptance.md` “PDF still recursive”
- `specs/033-page-lineage/` equality invariant
- OpenAPI `ChunkDetail.page_end` description
- `page_aware.rs` module docs (“MUST NOT span”)
