# 12 — Prompt & Eval Harness (Aug 2026)

## Prompt contract (product)

```text
CITATION RULES:
- After each factual claim, emit [N] matching a Document Chunk header.
- Do NOT write page numbers, filenames, or URLs. The system attaches those.
- Do NOT invent [N] outside the provided list.
- If the context does not support the claim, refuse — do not fabricate a cite.
```

Chunk header example:

```text
[1] (score: 0.850) page=12 doc="Q3 Report.pdf" modality=chart
…chunk text…
```

## Few-shot shapes

1. **Good:** short fact + `[1]`
2. **Multi:** fact spanning two chunks + `[1][2]`
3. **Refusal:** “Not answerable” / insufficient evidence, no `[N]`

## Acc / gold

Use `grounding_instructions_gold_compat()` — no markers; `strip_gold_citation_artifacts`.

## Eval harness (unfakable)

| Step | Action |
|------|--------|
| 1 | Seed fixture with known `page_start=4`, title `Fixture.pdf` |
| 2 | Scripted mock LLM returns: `The value is 42 [1]. See also [99] and page 999.` |
| 3 | Run rewrite |
| 4 | Assert: visible link text is `p.4`; markdown/HTML `title` is `Fixture.pdf`; href `page=4`; no `[99]`; href never contains `999`; prose `page 999` stripped |
| 5 | Playwright: click → `data-page="4"` |

Metrics to log when available: validity, locator accuracy.

## References (external, 2026)

- LLM Best Practices — RAG Citations (validate `[N]`; build deeplinks from metadata)
- Azure Architecture Center — RAG prompt engineering (cite Source N; metadata in context)
- DeepCitation / verifiable-rag — verification layer pattern (NLI = our P1)

## Cross-refs

- Prompt lens: [05-lenses/007-prompt-engineer.md](05-lenses/007-prompt-engineer.md)
- E2E: [08-e2e-test-matrix.md](08-e2e-test-matrix.md)
