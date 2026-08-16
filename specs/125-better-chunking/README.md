# SPEC-125 — Structure-Aware Markdown Chunking

> **Mission:** Stop heading-hard splits that emit orphan `##` chunks. Pack markdown to the token budget; repeat ATX hierarchy and table headers on continuations.  
> **Trigger:** 14 Aug 2026 — a short heading-dense markdown note became **4 chunks**, first chunk = heading only.

## Short verdict

| Layer | Finding |
|-------|---------|
| Symptom | Markdown files split on every ATX heading, no matter how small the section |
| Classification | **Strategy bug**, not a size-config miss — 1200 vs 600 does not pack siblings |
| Root cause | `extract_markdown_blocks` hard-splits; `MarkdownChunking` recursively chunks each block in isolation |
| Fix posture | Greedy packer (soft heading boundaries) + ATX prefix on continuations + table header repeat; tiktoken SSOT; kill switch |

```ascii
  .md / source_type=markdown
       │
       ├─ TODAY: heading HARD split → orphan ## chunk → N extract calls explode
       └─ TARGET: pack until token budget; split only when next block would overflow
                  continuation chunks carry # / ## / ### path in the TEXT
```

## Document map

```ascii
 00-why
   → 01-first-principles (LAW-125-1..10)
   → 02-cross-ref-matrix
   → 03-code-as-is
   → 04-target-architecture
   → 05-lenses/ (PO, fullstack, DB, UX, front, AI, markdown, RAG)
   → 06-ux-ui-spec
   → 07-implementation-plan
   → 08-test-protocol
   → 09-acceptance
   → 10-edge-cases
   → 11-research-evidence-aug-2026
   → 12-honest-assessment
```

## Status board

| ID | Item | Status |
|----|------|--------|
| D1 | Doc pack (00–12 + 8 lenses) | This tree |
| I1 | Fence-aware packer SSOT | Implementation |
| I2 | `MarkdownChunking` uses packer; ATX + table header | Implementation |
| I3 | tiktoken pack + `EDGEQUAKE_MARKDOWN_PACK` kill switch | Implementation |
| I4 | Langfuse `ingest.chunking` token distribution | Implementation |
| I5 | Workspace card honesty | Implementation |
| T1 | Heading-dense fixture + edge matrix + Acc unchanged + e2e + Playwright | Implementation |

## Config (unchanged cascade; packing is strategy behavior)

```ascii
  Document chunk_options  >  Workspace ChunkingPolicy  >  Fleet env  >  Default
       │
       └─ Tenant: ABSENT (SPEC-123 gap; not in v1)

  Markdown strategy auto for .md / source_type=markdown
  Packing default ON; EDGEQUAKE_MARKDOWN_PACK=0 restores heading-hard split
```

## Related

- [SPEC-026](../026-egdequake-vs-lightrag/) — markdown breadcrumbs / recursive cascade
- [SPEC-047](../047-rag-evaluation/) — atomic tables / MM blocks
- [SPEC-116](../116-adaptive-chunking/) — workspace size/overlap policy (orthogonal)
- [SPEC-123](../123-env-config-priority/) — Upload > Workspace > Tenant > Env (tenant not added here)
- [SPEC-124](../124-langfuse-support/) — ingest.chunking observation I/O

## Non-goals (v1)

- Tenant-level chunking
- LLM Anthropic contextual prefixes (`EDGEQUAKE_CONTEXTUAL_CHUNK` stays opt-in)
- Jina late-chunking
- Changing Recursive Acc `recursive_token_len`
- Auto-rebuild on save
- Setext / HTML `<h2>` parse (residual)
