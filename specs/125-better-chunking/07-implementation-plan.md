# 07 — Implementation Plan

## Phases

1. **Spec pack** — this tree  
2. **Packer SSOT** — `markdown_pack.rs`, fence-aware ATX walk, greedy pack, ATX prefix, table header repeat, `min_chunk_size`  
3. **Wire MarkdownChunking** — default pack ON; `EDGEQUAKE_MARKDOWN_PACK=0` legacy  
4. **Tokens + Langfuse** — tiktoken pack; distribution on `ingest.chunking`  
5. **UX copy** — workspace card hint  
6. **Tests** — heading-dense fixture, edges, Acc, e2e, Playwright, OTEL  

## File list

| File | Change |
|------|--------|
| `chunker/markdown_pack.rs` | New |
| `chunker/mod.rs` | `mod markdown_pack`; re-export if tests need |
| `chunker/markdown_chunking.rs` | Call packer |
| `table_preprocessor.rs` | Pub(crate) header/sep helpers if not already |
| `langfuse_meta.rs` | Distribution fields |
| `pipeline/processing.rs` | Record distribution |
| `rag_span.rs` / `record_observation_io` | Richer output JSON |
| `.env.example` | `EDGEQUAKE_MARKDOWN_PACK` |
| `workspace-chunking-card.tsx` | Hint |
| Tests listed in [08-test-protocol.md](08-test-protocol.md) | New |

## Edge-case matrix

See [10-edge-cases.md](10-edge-cases.md).

## Definition of done

See [09-acceptance.md](09-acceptance.md).
