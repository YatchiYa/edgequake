# Lens 006 — AI Engineer

## Stake (2026)

Industry consensus: **model cites a handle; system attaches locators; verification
is non-optional.** Trusting the LLM to author page numbers ships confident lies.

## Design

```ascii
  Index metadata (page, title, chunk_id)
       → Prompt labels [N] + doc title + page= (hints)
       → Model emits [N] only
       → Deterministic rewrite + strip unknown
       → Eval: coverage / validity / locator accuracy
```

## Metrics (log / test)

| Metric | Definition |
|--------|------------|
| Citation validity | `%` of `[N]` in answer that ∈ catalog |
| Locator accuracy | href `page` == catalog `page_start` |
| Coverage (soft) | factual sentences with ≥1 cite — optional v1 |

## Non-goals v1

- Extra judge LLM / NLI per claim (latency + cost)
- Quote-then-cite two-pass generation
- Fine-tuned citation model

## Acc

Gold path must not run rewriter (SPEC-082). Existing strip remains.

## Cross-refs

- Prompt: [007-prompt-engineer.md](007-prompt-engineer.md)
- Harness: [../12-prompt-harness.md](../12-prompt-harness.md)
