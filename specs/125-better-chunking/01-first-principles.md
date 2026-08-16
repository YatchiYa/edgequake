# 01 — First Principles (LAW-125)

## Domain

Markdown chunking is a **structure-aware packing** problem. Token size is a *budget*. Headings are *preferred split points*, not mandatory cuts.

```ascii
  SPEC-116:  how LARGE a chunk may be (workspace / env geometry)
  SPEC-125:  WHERE to cut markdown (pack vs heading-hard split)

  Orthogonal: packing does not replace ChunkingPolicy.
              A 600-token budget still packs the heading-dense fixture into one chunk.
```

## Axioms

1. Retrieval and extract both need **self-contained** chunks. A heading without body is not a unit of meaning.
2. Document structure (ATX, tables, fences) is a **constraint**, not a splitter.
3. Context stripped at chunk time cannot be recovered by the embedder. Prefix it in the text.
4. \(N\) (chunk count) multiplies extract LLM cost (SPEC-116 LAW-116-7). Heading-dense notes must not inflate \(N\).
5. Acc-fair Recursive geometry is a **publication invariant**. Markdown packing must not retune `recursive_token_len`.

## Laws

| ID | Law | Rationale |
|----|-----|-----------|
| **LAW-125-1** | Token budget is a **pack target**, not a heading hard-split | Heading-dense note: 1200 still yields 4 chunks |
| **LAW-125-2** | Headings are **soft** boundaries — split only when the next block would exceed budget | Structure-aware RAG 2026 |
| **LAW-125-3** | **No orphan heading chunks.** Pack parent+children. Floor = `min_chunk_size` except last remainder / atomic undersize | Embed noise + wasted extract |
| **LAW-125-4** | Continuation chunks **repeat ATX hierarchy in the text** (`#` / `##` / `###`), not only `heading_path` metadata | Anthropic contextual retrieval analog, zero LLM cost |
| **LAW-125-5** | Oversized tables: never split mid-row; **repeat header + separator** on every piece | Row without labels is uninterpretable |
| **LAW-125-6** | Packing token count = tiktoken `cl100k` SSOT (`token_estimator::count_tokens`) | Honest tokens vs word-count recursive |
| **LAW-125-7** | Acc-fair Recursive/Fixed geometry **unchanged**. Packing is Markdown-strategy behavior | SPEC-026 / SPEC-116 Acc pin |
| **LAW-125-8** | Fence-safe parse: ATX inside fenced code is not a heading. Atomic: fences, pipe tables, MM blocks | SPEC-047 |
| **LAW-125-9** | Future ingestions only. Rebuild required to re-chunk | LAW-116-4 |
| **LAW-125-10** | Observability: emit **actual** token distribution on `ingest.chunking` (min/p50/max, orphan count). Never dump chunk text | LAW-124-8 |
| **LAW-125-11** | Continuation overlap is **boundary overlap**: ATX path once, then the last full sentence of the previous body (capped at `chunk_overlap` tokens). Never a mid-sentence token slice. Fences re-open/close on overflow. MM blocks stay atomic even over budget. | Aug 2026 tokenizer-aware markdown SOTA |

## Causal diagram

```ascii
  markdown AST (ATX + atomic regions)
         │
         ▼
  ┌──────────────────┐
  │ Greedy packer    │  current + next <= budget → append
  │ LAW-125-1..3     │  else split oversized / emit + ATX prefix
  └────────┬─────────┘
           │
           ├─ table overflow → header+sep repeat     LAW-125-5
           ├─ prose overflow → recursive + ATX path  LAW-125-4
           └─ tokens = tiktoken                      LAW-125-6
           │
           ▼
        N chunks → extract → embed
        (N much smaller on heading-dense notes)
```

## Kill switch

```bash
# Default ON (unset / 1 / true / yes)
export EDGEQUAKE_MARKDOWN_PACK=0   # restore heading-hard split
```

## Config cascade (geometry — unchanged)

```ascii
  Document ChunkOptions  >  Workspace ChunkingPolicy  >  Fleet env  >  Default
  Tenant: not in v1 (honest gap vs SPEC-123)
```

## Acc-fair identity (unchanged)

```text
Recursive/Fixed(1200, 100)  ≡  EDGEQUAKE_ADAPTIVE_CHUNKING=0
                               + EDGEQUAKE_CHUNK_SIZE=1200
                               + EDGEQUAKE_CHUNK_OVERLAP=100
Markdown packing does NOT apply to ChunkStrategy::Recursive | Fixed | Pdf
```

## Cross-refs

- Why: [00-why.md](00-why.md)
- Architecture: [04-target-architecture.md](04-target-architecture.md)
- Research: [11-research-evidence-aug-2026.md](11-research-evidence-aug-2026.md)
