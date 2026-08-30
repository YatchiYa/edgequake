# Lens 007 — Prompt Engineer

## Product system instructions (delta)

In `grounding_instructions()`:

1. After each factual claim, cite with `[N]` matching chunk headers.
2. **Do not** write page numbers, filenames, or URLs — the system attaches them.
3. **Do not** invent `[N]` outside the provided list.
4. If unsupported, refuse; do not fabricate a cite.

## Few-shot (in harness doc / optional prompt appendix)

| Example | Expected |
|---------|----------|
| Good | Fact + `[1]` |
| Multi | Fact + `[1][2]` |
| Refusal | “Not answerable” with no `[N]` |

## Chunk header contract

```text
[1] (score: 0.850) page=12 doc="Q3 Report.pdf" modality=chart
```

Title is a **disambiguation hint**, not a license to copy into the answer.

## Acc / gold

`grounding_instructions_gold_compat()` unchanged: no citation markers.

## Cross-refs

- [../12-prompt-harness.md](../12-prompt-harness.md)
- `edgequake-query/src/grounding.rs`
