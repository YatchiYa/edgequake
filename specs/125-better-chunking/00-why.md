# 00 — Why SPEC-125

## Trigger

Partner report, 14 Aug 2026:

> The documents are flagged as markdown format, so they're being split on headings, apparently no matter how small the section is.
>
> A passage with a `##` parent and three `###` children is split into **4 chunks**, with the first one just being the `##` heading.

This is not a partner “chunk size too small” config miss. The markdown strategy treats every ATX heading as a **hard** split.

## Product WHY

```ascii
  User: “Why is this short note 4 chunks? The first chunk is just a heading.”
       │
       ▼
  Today:
       Workspace size/overlap (SPEC-116)  — ignored by heading-hard split
       Fleet EDGEQUAKE_CHUNK_SIZE=1200    — still 4 orphans
       Breadcrumb metadata                — extract prompt only; embed text is heading-only
              │
              ▼
  Blind spot: MarkdownChunking never PACKS sibling sections to the token budget
```

## Five WHYs

1. **Why 4 chunks from a short note?** Each ATX heading starts a new block; blocks are chunked in isolation.
2. **Why is the first chunk only the parent `##`?** The parent heading has no body before the first `###`; flush emits a heading-only block.
3. **Why didn’t raising chunk size help?** Size is applied *inside* each block, not across heading boundaries.
4. **Why are embeddings / extract worse?** Heading-only chunks have no entities; child chunks lose parent title in the stored text (`heading_path` is metadata, not prefix).
5. **Root cause:** heading-aware split copied LightRAG *section metadata* without a *packer*. Soft boundaries + ATX continuation prefixes are the 2026 RAG default.

## Job to be done

> When I ingest markdown, headings organize the document; they do not explode it. Sibling sections pack until the token budget. If a section or table must split, every continuation still carries `#` / `##` / `###` (and table headers) so a retrieved chunk is self-anchored.

## Success criteria

1. Heading-dense fixture (`##` + three `###`) → **1 packed chunk** (not 4) at product sizes 600/800/1200.
2. No heading-only chunk when a following body exists.
3. Continuation chunks repeat ATX path in the **text**.
4. Oversized tables: no mid-row split; header + separator repeated.
5. Acc-fair Recursive/Fixed geometry unchanged.
6. Kill switch `EDGEQUAKE_MARKDOWN_PACK=0` restores today’s split.
7. Langfuse `ingest.chunking` shows token min/p50/max + orphan count (no chunk text dump).

## Cross-refs

- Laws: [01-first-principles.md](01-first-principles.md)
- Code: [03-code-as-is.md](03-code-as-is.md)
- Edges: [10-edge-cases.md](10-edge-cases.md)
- Research: [11-research-evidence-aug-2026.md](11-research-evidence-aug-2026.md)
