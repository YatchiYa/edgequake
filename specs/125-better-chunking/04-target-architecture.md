# 04 — Target Architecture

## Packer SSOT

```ascii
  markdown
    │
    ├─ 1. atomic regions (fence / pipe table / mm)     LAW-125-8
    ├─ 2. heading walk on PLAIN regions only (ATX)
    │      skip lines inside fences
    │
    ▼
  greedy packer (markdown_pack.rs)
    tokens = tiktoken cl100k
    if current + next <= chunk_size  → append (soft boundary)
    else if current empty            → split oversized unit
         table  → row batches + repeated header+sep
         prose  → RecursiveCharacterChunking + ATX prefix on each piece
    else                             → emit current; start next
                                       (continuation: ATX path once + last sentence overlap)
    never emit heading-only if a following body exists
    honor min_chunk_size except last remainder / atomic undersize
    fence overflow → opener+closer on every piece
    MM overflow → stay atomic (do not shred figures)
```

## Strategy wiring

```ascii
  MarkdownChunking
       │
       ├─ EDGEQUAKE_MARKDOWN_PACK=0  → TODAY hard-split path (kill switch)
       └─ default ON                 → packer
              │
              ▼
         ChunkResult.content includes source ATX in packed windows
         Continuation pieces prepend:

            # Ancestor
            ## Parent
            ### Current

            <body continuation>
```

## Observability

```ascii
  ingest.chunking
    input:  {"chars":N}
    output: {"chunks":C,"token_min":A,"token_p50":B,"token_max":D,"orphan_heading_chunks":O}
    meta:   chunk_strategy, chunk_size, overlap  (existing)
            + token_min / token_p50 / token_max / orphan_heading_chunks
```

No chunk text (LAW-124-8).

## DRY / SOLID

| Principle | Application |
|-----------|-------------|
| SRP | Packer ≠ recursive Acc merge ≠ geometry policy |
| OCP | New markdown edge (setext) extends packer parse; Recursive untouched |
| DIP | `MarkdownChunking` depends on packer fn, not inline heading loops |
| DRY | Table header parse shared; tiktoken via `count_tokens` only |
| ISP | Kill switch is env on packer, not a second strategy enum |

## Out of packer

- `ChunkStrategy::Recursive` / `Fixed` / `Pdf` / `Semantic`
- Workspace/tenant size cascade
- LLM contextual prefixes
