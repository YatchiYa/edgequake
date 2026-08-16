# Lens 006 — AI Engineer

## Why packing beats heading-hard split

Extract LLMs and embedders see **chunk text**, not the original document tree. A `##` orphan:

- Wastes an extract call (empty or hallucinated entities)
- Embeds a title with no facts → retrieval noise
- Multiplies \(N\) on outline-style notes (SPEC-116: \(N \times y\) → \(M\))

## Why ATX in the text (not only metadata)

`heading_path` already reaches extract via `---Section Context---`. Embeddings and BM25/FTS do **not** see that block unless it is in `content`. Anthropic Contextual Retrieval (2024) showed prepended context cuts top-20 retrieval failures ~35–49%. Deterministic ATX lines are the zero-cost analog.

```ascii
  chunk 2 of oversized ### Holiday
  ┌─────────────────────────────────────┐
  │ # (none)                            │
  │ ## Personal Topics                  │
  │ ### Holiday Greetings & Festivities │
  │                                     │
  │ <rows 40–80 of section>             │
  └─────────────────────────────────────┘
```

## Token SSOT

Pack with tiktoken cl100k — same family as embedding/GPT-4 tokenizers. Do **not** use recursive word-count for Markdown packing (would under-count punctuation-heavy notes and disagree with `TextChunk.token_count`).

## Kill switch vs Acc

Acc publication uses Recursive/Fixed, not Markdown. Kill switch is for product rollback and for any Acc arm that *does* ingest `.md` via Markdown strategy.

## Non-goal v1

Per-chunk LLM “situate this chunk” (Anthropic Haiku). Existing `EDGEQUAKE_CONTEXTUAL_CHUNK` remains a separate embed preamble.
