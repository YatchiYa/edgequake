# 02 — Cross-Reference Matrix

## Spec / doc map

| Spec / Doc | Relationship to SPEC-125 |
|------------|--------------------------|
| [SPEC-026](../026-egdequake-vs-lightrag/) | Markdown breadcrumbs + recursive cascade; packing extends Markdown strategy |
| [SPEC-047](../047-rag-evaluation/) | Atomic fences / pipe tables / MM — packer must not split interiors |
| [SPEC-091](../091-simplify-data-layer/) | `EDGEQUAKE_CONTEXTUAL_CHUNK` preamble (orthogonal; stays opt-in) |
| [SPEC-116](../116-adaptive-chunking/) | Size/overlap policy; packing is independent of 600/800/1200 |
| [SPEC-123](../123-env-config-priority/) | Tenant layer **not** added; documented gap |
| [SPEC-124](../124-langfuse-support/) | `ingest.chunking` I/O + `IngestKgMeta`; add distribution keys |
| Heading-dense report | Heading-only first chunk on `##` + three `###` |

## Violation / gap register

| ID | Gap | Law | Fix |
|----|-----|-----|-----|
| G1 | ATX heading is a hard split | LAW-125-1,2 | Greedy packer |
| G2 | Heading-only parent block emitted | LAW-125-3 | Never emit orphan if body follows |
| G3 | `format_breadcrumb` computed, unused in chunk text | LAW-125-4 | ATX prefix on continuations (and packed first chunk keeps source ATX) |
| G4 | `min_chunk_size` ignored by Markdown/Recursive | LAW-125-3 | Honor in packer |
| G5 | Table overflow loses header | LAW-125-5 | Repeat header+sep (DRY with preprocessor) |
| G6 | ATX inside fences treated as headings | LAW-125-8 | Fence-aware walk |
| G7 | Three token estimators | LAW-125-6 | Pack with tiktoken; leave recursive word-count for Acc |
| G8 | Langfuse records target size, not emitted distribution | LAW-125-10 | min/p50/max + orphan count |
| G9 | Tenant not in chunking cascade | SPEC-123 | Honest non-goal v1 |
| G10 | Workspace card silent on markdown packing | LAW-116 honesty | Copy + future-only |

## ASCII dependency

```ascii
  SPEC-026 (markdown IR + recursive)
       │
       ├─ SPEC-047 (atomic regions)
       ├─ SPEC-116 (geometry policy)
       └─ SPEC-125 (this) packer on Markdown strategy
              ├─ constrains MarkdownChunking only
              ├─ extends ingest.chunking meta (SPEC-124)
              └─ does NOT change Recursive Acc token_len
```

## Code SSOT (target)

| Concern      | Path                                                      |
| --------------| -----------------------------------------------------------|
| Packer       | `edgequake-pipeline/src/chunker/markdown_pack.rs`         |
| Strategy     | `chunker/markdown_chunking.rs`                            |
| Heading IR   | `markdown_ir/parse.rs` (fence-aware or packer-local walk) |
| Atomic       | `chunker/atomic_blocks.rs`                                |
| Table header | share with `table_preprocessor.rs`                        |
| Tokens       | `token_estimator.rs`                                      |
| Kill switch  | `markdown_pack.rs` / env `EDGEQUAKE_MARKDOWN_PACK`        |
| KG meta      | `edgequake-observability/src/langfuse_meta.rs`            |
| Ingest span  | `pipeline/processing.rs` `chunk_under_span`               |
| UI           | `workspace-chunking-card.tsx`                             |

## DRY rule

Packer math lives **only** in `markdown_pack.rs`. Recursive Acc merge stays in `recursive.rs`. UI never reimplements packing.

## External refs

- https://docs.langchain.com/oss/python/integrations/splitters/markdown_header_metadata_splitter
- https://www.anthropic.com/engineering/contextual-retrieval
- https://ai-tldr.dev/learn/rag/chunking-and-ingestion/chunk-code-tables-markdown/
- https://arxiv.org/abs/2409.04701 (late chunking — non-goal v1)
