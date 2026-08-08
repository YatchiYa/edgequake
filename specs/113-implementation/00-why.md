# 00 — Why SPEC-113

## Trigger

GitHub issue [#369](https://github.com/raphaelmansuy/edgequake/issues/369) (opened 2026-08-07 by @ravimohta):

> Edgequake invokes ollama with thinking parameters if the llm name has the word "qwen3" in it.

Reporter configures a **Qwen3 vision-language** variant (e.g. `qwen3-vl-*`) as the Ollama chat model with no explicit `reasoning_effort`. Every chat request fails with Ollama’s `"does not support thinking"` class error — until they rename the model so the string no longer contains `qwen3`.

## User impact

| Layer | Impact if ignored |
|-------|-------------------|
| Reliability | Hard failure on **every** chat / query / extract call against false-positive “thinking” names |
| Vision / PDF | VL models under the `qwen3*` family are exactly the ones operators pick for multimodal ingest — highest blast radius |
| Trust | Workaround is `ollama cp … vl-instruct-8b` — proves the product is guessing from **names**, not capabilities |
| Discoverability | Discovery already reports `supports_thinking` from `/api/tags` — chat path ignores that truth |
| Support cost | “Ollama broken / model broken” tickets when only EdgeQuake’s auto-`think` injection is wrong |

## Why this pack (not a one-line `!contains("-vl")`)

1. **Code is law** — the defect is a substring heuristic in `OllamaProvider::is_thinking_model`, used by `resolve_think` on the Auto path. Special-casing `-vl` recreates the same class of bug for the next family rename.
2. **First principle** — Ollama already publishes a per-model `capabilities` array (`thinking` ∈ list). Guessing from the model id violates the crate’s own discovery contract (“zero heuristics”).
3. **Dual surface** — `reasoning_capabilities` also name-matches `qwen` for Ollama effort clamping. Fix both or the Auto / clamp paths stay inconsistent.
4. **Product promise** — “any Ollama model I configure should work” ([#369](https://github.com/raphaelmansuy/edgequake/issues/369) expected behavior). Capability-gated `think` is the only durable way to keep that promise.

## Non-goals

- Changing Ollama server / model templates (upstream; out of product control).
- Redesigning SPEC-109 role hierarchy or UI effort pickers (reuse; do not fork).
- Claiming every Qwen3-VL build is non-thinking forever — **capabilities** decide, not folklore.
- Implementing Waves A–E inside this documentation deliverable.

## Success condition

- Engineering has LAW-113 → code symbols → tests mapping ([`02-cross-ref-matrix.md`](02-cross-ref-matrix.md)).
- Fix train is DRY/SOLID with fail-safe Auto semantics ([`04-fix-plan.md`](04-fix-plan.md)).
- Edge cases (aliases, stale cache, old Ollama, explicit effort, VL) are enumerated and gated ([`06-edge-cases.md`](06-edge-cases.md), [`05-e2e-test-matrix.md`](05-e2e-test-matrix.md)).
- Partners can unblock today via ops workaround ([`07-ops-runbook.md`](07-ops-runbook.md)) without waiting for a release.
- Brutal honesty states what #369 proves and what it does not ([`measurements/BRUTAL-HONESTY.md`](measurements/BRUTAL-HONESTY.md)).
