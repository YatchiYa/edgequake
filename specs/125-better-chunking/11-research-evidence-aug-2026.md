# 11 — Research Evidence (August 2026)

## Claim → evidence → EdgeQuake

| ID | Claim | Evidence | EQ meaning |
|----|-------|----------|------------|
| R1 | Header orphaning is a known splitter failure | LangChain MarkdownHeaderTextSplitter troubleshooting: tiny first chunk from headers; strip or pack | Heading-dense note is this bug |
| R2 | Structure-aware pack at document tree beats token arithmetic alone | n1n.ai 2026-08-09 production chunking; mdstill token-aware markdown walk | Greedy packer |
| R3 | Never split tables mid-row; repeat header on every piece | AI/TLDR markdown/tables 2026; EQ `table_preprocessor` already repeats for table-dominant docs | LAW-125-5 |
| R4 | Prepending context to chunks cuts retrieval failures | Anthropic Contextual Retrieval (2024): embeddings −35% fail@20; +BM25 −49% | Deterministic ATX prefix v1 |
| R5 | Late chunking is a different layer | Jina arXiv:2409.04701 — pool token vectors after full-doc encode | Non-goal v1 |
| R7 | Boundary overlap beats mid-sentence token overlap | Tokenizer-aware markdown packing (DEV, 2026): last heading + last full sentence of chunk N lead chunk N+1 | LAW-125-11 |
| R8 | H1/title-chain on every continuation | structchunk / breadchunks: breadcrumb in chunk text, not metadata only | LAW-125-4 |
| R9 | Fence pieces must re-emit opener | DEV 2026 `_split_code` re-opens language fence on every fragment | LAW-125-11 |
| R10 | Atomic MM/figures may exceed budget | mdstill: never tear atomic blocks | LAW-125-8 |

## Bibliography

1. LangChain — Markdown header splitter — https://docs.langchain.com/oss/python/integrations/splitters/markdown_header_metadata_splitter  
2. Anthropic — Contextual Retrieval — https://www.anthropic.com/engineering/contextual-retrieval  
3. AI/TLDR — Chunking code, tables, and markdown — https://ai-tldr.dev/learn/rag/chunking-and-ingestion/chunk-code-tables-markdown/  
4. n1n.ai — RAG chunking strategies that survive production (2026-08-09) — https://explore.n1n.ai/blog/rag-chunking-strategies-production-beyond-512-tokens-2026-08-09  
5. mdstill — Token-aware chunking — https://mdstill.com/blog/token-aware-chunking-for-rag  
8. Tokenizer-aware markdown packing (DEV, 2026) — https://dev.to/gabrielanhaia/tokenizer-aware-markdown-chunking-that-doesnt-shred-tables-3kd7  
9. structchunk — hierarchical + linear markdown packers with header breadcrumbs — https://github.com/yzp0111/structchunk  
10. breadchunks — parent absorption, ATX-only — https://github.com/jonathanong/breadchunks

## Causal synthesis

```ascii
  heading-hard split ──► orphan ## ──► N↑ ──► extract $↑ + embed noise
  structure pack     ──► self-contained chunks ──► N↓ on outlines
  ATX prefix         ──► embed/FTS see hierarchy without LLM contextualizer
```
