# Lens 005 — AI Engineer

## Stake

Extract quality tracks **chunk geometry**. Underfilled 230-token chunks waste
context windows on heading-only / figure-only jobs and duplicate MM text.
Over-packing (dropping structure) would mix unrelated sections and hurt
entity/relation extract.

## Geometry target

```ascii
  Budget = workspace chunk_size (typically 1200 tiktoken)
  Fill   = token_p50 / budget     target ≥ 0.55 on ≥8k docs
  Atomic interiors stay whole (tables, VLM blocks, fences)
  ATX prefix on continuations     (already LAW-125-4, reused)
```

## MM once (extract impact)

Duplicate `[Chart Name]` chunks cause:

- double entity mentions for the same figure
- extract jobs that see only the sidecar, not surrounding methods prose
- Acc noise (SPEC-047 already flagged this)

Skip sidecar when Pass-A inlined VLM. If VLM was **not** inlined (failed
analyze item), sidecar remains — that is the LightRAG **intent**.

## Prompt / model

No Pass-A / extract prompt change in v1. No new LLM call for “situate this
chunk” (Anthropic contextual retrieval analog is ATX prefix, already in 125).

`EDGEQUAKE_CONTEXTUAL_CHUNK` stays opt-in and orthogonal.

## Kill vs Acc

Acc PDF dual-SUT: either pin `EDGEQUAKE_PDF_PACK=0` or re-score after WP-2.
Do not hide PDF N drift in publication notes.

## Cross-refs

- Research: [../11-research-evidence-aug-2026.md](../11-research-evidence-aug-2026.md)
- Honest Acc: [../12-honest-assessment.md](../12-honest-assessment.md)
