# SPEC-109 — Why

## Five WHYs

1. **Why do GPT-5-mini / GPT-5.4-* ingest runs produce empty or truncated JSON?**  
   Because reasoning models spend a large share of `max_completion_tokens` on internal chain-of-thought before the visible answer — structured extraction and vision captions starve.

2. **Why isn't setting `reasoning_effort="none"` enough today?**  
   (a) Native `OpenAIProvider` in `edgequake-llm` never forwards the field. (b) Some models (notably `gpt-5-mini`) **reject** `"none"` and only accept `minimal|low|medium|high` — a hardcoded `"none"` becomes a 400.

3. **Why not only fix the provider and keep the hardcode?**  
   Partners need different tradeoffs: extract/vlm want budget for output; hard RAG queries may want `medium`/`high`. That requires **configuration**, not a single global pin.

4. **Why role-scoped (extract / vlm / query / …) instead of one fleet dial?**  
   Ingest paths are high-volume and schema-sensitive; query paths are quality-sensitive. One dial forces a bad compromise. Existing `LlmRole` / `llm_roles` metadata is the DRY place to hang effort.

5. **Why capability clamp in `edgequake-llm` rather than EdgeQuake?**  
   Every provider already maps `CompletionOptions.reasoning_effort`. Model acceptance sets change with vendor releases. One registry avoids UI/API/extractor drift and 400s (SOLID + DRY).

## Causal chain

```text
Partner uses GPT-5 mini for vision + extract
  → default / omitted reasoning_effort (often medium)
  → reasoning tokens consume completion budget
  → empty / truncated JSON · failed extract · weak captions
  → "next version: make effort configurable"

Today's partial mitigation:
  extraction_completion_options → reasoning_effort="none"
    → native OpenAI drops field          (F1)
    → gpt-5-mini rejects "none"          (F2 / E2E-109-02)
    → no workspace / query / tenant UI   (F3)
```

## Relation to sibling specs

| Spec | Relationship |
|------|----------------|
| Pipeline `completion_options` / SPEC-047 notes | Owns current `"none"` + Mistral Large omit — SPEC-109 **replaces hardcode** with resolved+clamped effort |
| [SPEC-103](../103-llm-cache/) | Cache keys must incorporate effort when present (edge case) |
| [SPEC-108](../108-extraction-compared-light-rag/) | Density/chunking; orthogonal — do not fold effort into 108 |
| [SPEC-043](../043-update-edgequake-llm/) | Crate bump / catalog sync vehicle for Wave 0 |

## Non-goals (this pack)

- Rewriting Acc protocol  
- Switching OpenAI path to Responses API in v1  
- Showing CoT / thinking traces in the WebUI  
- Per-chunk adaptive effort
