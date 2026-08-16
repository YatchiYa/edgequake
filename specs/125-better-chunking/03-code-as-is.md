# 03 — Code As-Is

## Markdown path

```ascii
  admission: .md / mime markdown / source_type=markdown
       │
       ▼
  ChunkStrategy::Markdown
       │
       ▼
  MarkdownChunking::chunk
       ├─ maybe_induce_structure (FAQ env)
       ├─ extract_markdown_blocks   ← HARD split every ATX line
       └─ RecursiveCharacterChunking.chunk(block) per block
            └─ no min_chunk_size
            └─ recursive_token_len = WORD COUNT (not tiktoken)
```

Heading-dense fixture:

```ascii
  ## Parent Topics\n\n            → block 1 (orphan heading)
  ### First Child …\n body        → block 2
  ### Second Child …\n body       → block 3
  ### Third Child …\n body        → block 4
```

`format_breadcrumb` is bound to `_breadcrumb` and never prepended.

## Geometry resolve (SPEC-116 — still true)

```ascii
  EDGEQUAKE_ADAPTIVE_CHUNKING (default ON)
       │
       ├─ ON  → 1200 / 800 / 600 by bytes + ~8.3% overlap
       └─ OFF → EDGEQUAKE_CHUNK_SIZE / OVERLAP (1200/100)
       │
       ▼
  workspace ChunkingPolicy (Inherit | Adaptive | Fixed)
       │
       ▼
  document ChunkOptions last
```

No tenant. Markdown strategy ignores packing regardless of those sizes.

## Token estimators (three)

| Site | Function | Meaning |
|------|----------|---------|
| `token_estimator::count_tokens` | tiktoken cl100k | Claimed SSOT; `TextChunk.token_count` |
| `recursive_token_len` | words / CJK chars÷1.5 | Recursive merge budget (Acc) |
| `TokenBasedChunking` | `chunk_size * 4` chars | Fixed strategy |
| `section_context.rs` | `len/4` | Extract breadcrumb truncate |

## What already exists

| Layer | Present? |
|-------|----------|
| Heading IR + `heading_path` metadata | Yes |
| Extract `---Section Context---` | Yes (prompt only) |
| Table preprocessor (table-dominant docs) | Yes — repeats header per group |
| Atomic fences/tables/MM | Yes — Recursive only, after heading split |
| `min_chunk_size` default 100 | Declared; Markdown/Recursive ignore |
| Contextual chunk env | Yes — generic preamble, off by default |
| Langfuse `ingest.chunking` | chars in / chunk count out; target size/overlap/strategy |

## Gap

Partner cannot get LightRAG-like packing on markdown without switching strategy to Recursive (and losing heading metadata). Size knobs cannot fix heading-hard splits.
