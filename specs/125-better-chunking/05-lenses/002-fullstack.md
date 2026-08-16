# Lens 002 — Full Stack Developer

## SOLID / DRY

| Principle | Application |
|-----------|-------------|
| SRP | `markdown_pack` packs; `markdown_ir` describes headings; `adaptive_chunking` sizes |
| OCP | Kill switch wraps packer; no second `ChunkStrategy` |
| DIP | Tests call packer + `MarkdownChunking`, not private flush closures |
| DRY | One tiktoken call site for packing; table header helper shared |

## Touch list

1. `chunker/markdown_pack.rs` (new SSOT)
2. `chunker/markdown_chunking.rs` — call packer / kill switch
3. Fence-aware heading walk (packer or `markdown_ir/parse.rs`)
4. `table_preprocessor.rs` — export header/separator helpers if needed
5. `token_estimator::count_tokens`
6. `langfuse_meta.rs` + `processing.rs` `chunk_under_span`
7. `.env.example` `EDGEQUAKE_MARKDOWN_PACK`
8. `workspace-chunking-card.tsx` copy
9. Tests: pipeline unit + e2e + Playwright + OTEL

## Anti-patterns

- Packing inside Recursive (breaks Acc)
- Re-implementing tiktoken as words
- Prepending breadcrumbs only to extract prompts (already done; embed still orphan)
- Reading env in the React card
- Dumping chunk text to Langfuse
